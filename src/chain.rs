//! Decapsulation chain with a fixed public schedule, and the key-generation
//! chain.  See RESEARCH note §4: the eigen-structure forces
//!   graph, prodker, glue, rich x (e-6), split, prodker, graph
//! and key generation selects keys for which this schedule holds, so the
//! sequence of operations never depends on the secret key or the ciphertext.

use crate::cm::mul_vartime;
use crate::ct::{Choice, ConditionallySelectable};
use crate::field::Field;
use crate::kani::{div_dbl_iter, div_dbl_iter_batch, Div, Glue, Richelot, Split};
use crate::mont::{xdbl, ProjCurve, XPoint};
use crate::strategy::optimal;
use crate::theta::{product_isogeny, product_isogeny_with_kernel_images};
use crate::weier::{complementary_2_torsion_xs, Isog2, ToMontgomery, WCurve, WIso, WPoint};
use num_bigint::BigUint;

pub type Pair<F> = (WPoint<F>, WPoint<F>);
pub type KeygenOut<F> = (WCurve<F>, WCurve<F>, Vec<Pair<F>>);

pub fn graph_identity<F: Field>(e: &WCurve<F>, p: &Pair<F>) -> Pair<F> {
    (e.add(&p.0, &p.1), e.sub(&p.0, &p.1))
}

/// Product-kernel step: each factor gets one 2-torsion point of the kernel,
/// selected in constant time (exactly one of the two generators is nonzero
/// on each factor for honest inputs).
fn curve_is_mont<F: Field>(e: &WCurve<F>) -> bool {
    bool::from(e.a6.is_zero()) && bool::from(e.a4.ct_eq(&F::one()))
}

/// [2^n]P on a Montgomery curve, x-only. The returned point carries the
/// affine x and the infinity flag; its y is unused (callers need the kernel
/// x-coordinate only).
fn mont_xdbl_x<F: Field>(e: &WCurve<F>, p: &WPoint<F>, n: usize) -> WPoint<F> {
    let c = ProjCurve::from_affine(e.a2);
    let mut x = XPoint {
        x: p.x,
        z: F::conditional_select(&F::one(), &F::zero(), p.inf),
    };
    for _ in 0..n {
        x = xdbl(&x, &c);
    }
    let inf = x.is_infinity();
    let r = WPoint {
        x: x.affine(),
        y: F::zero(),
        inf: Choice::from(0),
    };
    WPoint::conditional_select(&r, &WPoint::infinity(), inf)
}

pub fn prodker_step<F: Field + Send + Sync>(
    e1: &WCurve<F>,
    e2: &WCurve<F>,
    k1: &Pair<F>,
    k2: &Pair<F>,
    h: usize,
) -> (Isog2<F>, Isog2<F>) {
    let mont = curve_is_mont(e1) && curve_is_mont(e2);
    let (a1, a2, b1, b2) = if mont && h >= 48 {
        std::thread::scope(|scope| {
            let a1 = scope.spawn(|| mont_xdbl_x(e1, &k1.0, h));
            let a2 = scope.spawn(|| mont_xdbl_x(e1, &k2.0, h));
            let b1 = scope.spawn(|| mont_xdbl_x(e2, &k1.1, h));
            let b2 = scope.spawn(|| mont_xdbl_x(e2, &k2.1, h));
            (
                a1.join().expect("prodker"),
                a2.join().expect("prodker"),
                b1.join().expect("prodker"),
                b2.join().expect("prodker"),
            )
        })
    } else if mont {
        (
            mont_xdbl_x(e1, &k1.0, h),
            mont_xdbl_x(e1, &k2.0, h),
            mont_xdbl_x(e2, &k1.1, h),
            mont_xdbl_x(e2, &k2.1, h),
        )
    } else {
        (
            e1.dbl_iter(&k1.0, h),
            e1.dbl_iter(&k2.0, h),
            e2.dbl_iter(&k1.1, h),
            e2.dbl_iter(&k2.1, h),
        )
    };
    let kap1 = WPoint::conditional_select(&a1, &a2, a1.inf);
    let kap2 = WPoint::conditional_select(&b1, &b2, b1.inf);
    (Isog2::new(e1, &kap1.x), Isog2::new(e2, &kap2.x))
}

pub fn map_pair<F: Field>(f: &(Isog2<F>, Isog2<F>), p: &Pair<F>) -> Pair<F> {
    (f.0.eval(&p.0), f.1.eval(&p.1))
}

/// x-coordinates (x(P), x(Q), x(P+Q)) of a 2-torsion basis, the third from
/// the root sum of x^3 + a2 x^2 + ...
pub fn two_torsion_xs<F: Field>(e: &WCurve<F>, p: &WPoint<F>, q: &WPoint<F>) -> [F; 3] {
    [p.x, q.x, -e.a2 - p.x - q.x]
}

/// Jacobian phase: `r` Richelot steps along the kernel <D1, D2> (order >= 2^r),
/// traversed with an optimal strategy; the generators and `extra` divisors
/// are pushed through every step.  Returns the final h.
pub fn jacobian_phase<F: Field>(
    h0: [F; 7],
    gens: &mut [Div<F>; 2],
    order_log: usize,
    r: usize,
    extra: &mut [Div<F>],
) -> [F; 7] {
    let mut h = h0;
    if r == 0 {
        return h;
    }
    let strategy = optimal(r, 0.6, 1.0);
    // leaf generator of order 2^r
    let g0v = div_dbl_iter_batch(&h, &gens[..], order_log - r);
    let mut stack: Vec<([Div<F>; 2], usize)> = vec![([g0v[0], g0v[1]], 0)];
    let mut idx = 0;
    for row in 0..r {
        loop {
            let (q, k) = *stack.last().unwrap();
            if k >= r - 1 - row {
                break;
            }
            let s = strategy[idx];
            idx += 1;
            let m = div_dbl_iter_batch(&h, &q, s);
            stack.push(([m[0], m[1]], k + s));
        }
        let (leaf, _) = stack.pop().unwrap();
        let phi = Richelot::new(&h, &leaf[0], &leaf[1]);
        // push everything through in one batch
        let mut all: Vec<Div<F>> = Vec::with_capacity(2 * stack.len() + 2 + extra.len());
        for e in stack.iter() {
            all.push(e.0[0]);
            all.push(e.0[1]);
        }
        all.push(gens[0]);
        all.push(gens[1]);
        all.extend_from_slice(extra);
        let mapped = phi.map_batch(&all);
        let mut it = mapped.into_iter();
        for e in stack.iter_mut() {
            e.0 = [it.next().unwrap(), it.next().unwrap()];
        }
        gens[0] = it.next().unwrap();
        gens[1] = it.next().unwrap();
        for d in extra.iter_mut() {
            *d = it.next().unwrap();
        }
        h = phi.hnew;
    }
    h
}

/// The decapsulation (2^e, 2^e)-chain on E x E with kernel <K1, K2> and the
/// fixed schedule.  Returns the codomain curve (both factors are the same
/// curve after the final graph step) and the images of `payload`.
pub fn decaps_chain<F: Field + Send + Sync>(
    ec: &WCurve<F>,
    k1: &Pair<F>,
    k2: &Pair<F>,
    e: usize,
    payload: &[Pair<F>],
) -> (WCurve<F>, Vec<Pair<F>>) {
    assert!(e >= 8);
    // step 1: graph of the identity  (x, y) -> (x + y, x - y)
    let mut k1 = graph_identity(ec, k1);
    let mut k2 = graph_identity(ec, k2);
    let mut pay: Vec<Pair<F>> = payload.iter().map(|p| graph_identity(ec, p)).collect();
    // step 2: product kernel (kernel now of order 2^(e-1))
    let f = prodker_step(ec, ec, &k1, &k2, e - 2);
    k1 = map_pair(&f, &k1);
    k2 = map_pair(&f, &k2);
    pay = pay.iter().map(|p| map_pair(&f, p)).collect();
    let (c1, c2) = (f.0.codomain, f.1.codomain);
    // step 3: gluing (kernel of order 2^(e-2))
    let a1 = c1.dbl_iter(&k1.0, e - 3);
    let a2 = c1.dbl_iter(&k2.0, e - 3);
    let b1 = c2.dbl_iter(&k1.1, e - 3);
    let b2 = c2.dbl_iter(&k2.1, e - 3);
    let g = Glue::new(two_torsion_xs(&c1, &a1, &a2), two_torsion_xs(&c2, &b1, &b2));
    let mut gens = [g.map(&k1.0, &k1.1), g.map(&k2.0, &k2.1)];
    let mut extra: Vec<Div<F>> = pay.iter().map(|p| g.map(&p.0, &p.1)).collect();
    // steps 4 .. e-3: Richelot (kernel of order 2^(e-3) after gluing)
    let r = e - 6;
    let h = jacobian_phase(g.h, &mut gens, e - 3, r, &mut extra);
    // step e-2: splitting (kernel of order 2^3 now; its 2-torsion is 4*gens)
    let s = Split::new(
        &h,
        &div_dbl_iter(&h, &gens[0], 2),
        &div_dbl_iter(&h, &gens[1], 2),
    );
    let mut k1 = s.map(&gens[0]);
    let mut k2 = s.map(&gens[1]);
    let mut pay: Vec<Pair<F>> = extra.iter().map(|d| s.map(d)).collect();
    let (e3, e4) = (s.e1, s.e2);
    // step e-1: product kernel (kernel of order 4)
    let f = prodker_step(&e3, &e4, &k1, &k2, 1);
    k1 = map_pair(&f, &k1);
    k2 = map_pair(&f, &k2);
    let _ = k2;
    pay = pay.iter().map(|p| map_pair(&f, p)).collect();
    let (e5, e6) = (f.0.codomain, f.1.codomain);
    // step e: graph of an isomorphism e6 -> e5 on the 2-torsion
    let tau = WIso::find(&e6, &e5);
    let _ = k1;
    let out: Vec<Pair<F>> = pay
        .iter()
        .map(|p| {
            let t = tau.eval(&p.1);
            (e5.add(&p.0, &t), e5.sub(&p.0, &t))
        })
        .collect();
    (e5, out)
}

/// Push a complementary domain 2-torsion point through `phi` to get a
/// rational 2-torsion x-coordinate on the (generally non-Montgomery)
/// codomain, used to put that codomain into Montgomery form for theta.
fn codomain_torsion_x<F: Field>(domain: &WCurve<F>, phi: &Isog2<F>) -> F {
    let xs = complementary_2_torsion_xs(domain, phi.x0);
    let img0 = phi.eval(&WPoint::new(xs[0], F::zero()));
    let img1 = phi.eval(&WPoint::new(xs[1], F::zero()));
    // Exactly one of the two non-kernel 2-torsion points maps to a finite
    // 2-torsion on the codomain under a separable 2-isogeny (the other may
    // coincide with the kernel of the dual); pick a finite image.
    F::conditional_select(&img1.x, &img0.x, !img0.inf)
}

/// Theta-coordinate decapsulation chain: same outer schedule and signature
/// as [`decaps_chain`], but the long middle (glue → (2,2)-steps → split) is
/// evaluated with [`product_isogeny`] in the level-2 theta model.
///
/// After the first product-kernel step the remaining kernel generators have
/// order `2^{e-2}` (the gluing input of the Mumford schedule).  The theta
/// routines expect lifts of order `2^{n+2}` for an `n`-step chain (the extra
/// two bits supply the 8-torsion used by gluing), so we take `n = e - 4`.
/// The Velu `Isog2` codomains are rewritten into Montgomery models before
/// gluing — the source formulas assume that shape.
///
/// Mumford [`decaps_chain`] is left unchanged; this is the parallel entry
/// point for Eigen-Séta decapsulation experiments / a future recover switch.
pub fn decaps_chain_theta<F: Field + Send + Sync>(
    ec: &WCurve<F>,
    k1: &Pair<F>,
    k2: &Pair<F>,
    e: usize,
    payload: &[Pair<F>],
) -> (WCurve<F>, Vec<Pair<F>>) {
    assert!(e >= 8);
    // step 1: graph of the identity  (x, y) -> (x + y, x - y)
    let mut k1 = graph_identity(ec, k1);
    let mut k2 = graph_identity(ec, k2);
    let mut pay: Vec<Pair<F>> = payload.iter().map(|p| graph_identity(ec, p)).collect();
    // step 2: product kernel (kernel now of order 2^(e-1))
    let f = prodker_step(ec, ec, &k1, &k2, e - 2);
    k1 = map_pair(&f, &k1);
    k2 = map_pair(&f, &k2);
    pay = pay.iter().map(|p| map_pair(&f, p)).collect();

    // Put both Isog2 codomains into Montgomery form (theta gluing/splitting
    // is derived for y^2 = x^3 + A x^2 + x).
    let (tm1, tm2) = std::thread::scope(|scope| {
        let a =
            scope.spawn(|| ToMontgomery::from_torsion(&f.0.codomain, codomain_torsion_x(ec, &f.0)));
        let b =
            scope.spawn(|| ToMontgomery::from_torsion(&f.1.codomain, codomain_torsion_x(ec, &f.1)));
        (a.join().expect("montgomery"), b.join().expect("montgomery"))
    });
    let map_mont = |p: &Pair<F>| (tm1.eval(&p.0), tm2.eval(&p.1));
    k1 = map_mont(&k1);
    k2 = map_mont(&k2);
    pay = pay.iter().map(map_mont).collect();
    let (c1, c2) = (tm1.curve, tm2.curve);

    // steps 3 .. e-2: (2^{e-4}, 2^{e-4})-isogeny in the theta model.  The
    // order-4 images of the kernel generators, needed by the trailing
    // product-kernel step, come back after the payload.
    let n = e - 4;
    let strategy = optimal(n, 1.0, 1.0);
    let pay_len = pay.len();
    let ((e3, e4), outs) =
        product_isogeny_with_kernel_images(&(c1, c2), &k1, &k2, &pay, n, &strategy);
    let mut pay = outs[..pay_len].to_vec();
    let mut k1 = outs[pay_len];
    let mut k2 = outs[pay_len + 1];

    // step e-1: product kernel (kernel of order 4)
    let f = prodker_step(&e3, &e4, &k1, &k2, 1);
    k1 = map_pair(&f, &k1);
    k2 = map_pair(&f, &k2);
    let _ = k2;
    pay = pay.iter().map(|p| map_pair(&f, p)).collect();
    let (e5, e6) = (f.0.codomain, f.1.codomain);
    // step e: graph of an isomorphism e6 -> e5 on the 2-torsion
    let tau = WIso::find(&e6, &e5);
    let _ = k1;
    let out: Vec<Pair<F>> = pay
        .iter()
        .map(|p| {
            let t = tau.eval(&p.1);
            (e5.add(&p.0, &t), e5.sub(&p.0, &t))
        })
        .collect();
    (e5, out)
}

/// What a variable-time walk of the decapsulation schedule observed.  Used
/// by the cryptanalysis module to test whether a wrong key "fails early"
/// (which would localise the secret) or only at the end.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainReport {
    /// The product step found exactly one kernel generator on each factor,
    /// as the honest schedule assumes.
    pub prodker_ok: bool,
    /// The gluing step received four finite 2-torsion points with no graph
    /// structure, as the honest schedule assumes.
    pub glue_ok: bool,
    /// First Richelot step (0-based, of `e - 6`) whose kernel determinant
    /// vanished, i.e. an unscheduled splitting.  `None` if none did.
    pub first_zero_delta_step: Option<usize>,
    /// Number of Richelot steps actually walked.
    pub richelot_steps: usize,
    /// The scheduled splitting step had a vanishing determinant.
    pub final_split: bool,
}

/// Walk the decapsulation schedule in variable time with structural checks
/// at every step (see `ChainReport`).  Doublings are done naively, so the
/// cost is quadratic in `e`; this is analysis code, not the KEM.
pub fn inspect_chain<F: Field + Send + Sync>(
    ec: &WCurve<F>,
    k1: &Pair<F>,
    k2: &Pair<F>,
    e: usize,
) -> ChainReport {
    use crate::kani::kernel_determinant;
    assert!(e >= 8);
    let fin = |p: &WPoint<F>| !bool::from(p.inf);
    let mut k1 = graph_identity(ec, k1);
    let mut k2 = graph_identity(ec, k2);
    // product step
    let a1 = ec.dbl_iter(&k1.0, e - 2);
    let a2 = ec.dbl_iter(&k2.0, e - 2);
    let b1 = ec.dbl_iter(&k1.1, e - 2);
    let b2 = ec.dbl_iter(&k2.1, e - 2);
    let prodker_ok = (fin(&a1) ^ fin(&a2)) && (fin(&b1) ^ fin(&b2));
    let f = prodker_step(ec, ec, &k1, &k2, e - 2);
    k1 = map_pair(&f, &k1);
    k2 = map_pair(&f, &k2);
    let (c1, c2) = (f.0.codomain, f.1.codomain);
    // gluing step
    let a1 = c1.dbl_iter(&k1.0, e - 3);
    let a2 = c1.dbl_iter(&k2.0, e - 3);
    let b1 = c2.dbl_iter(&k1.1, e - 3);
    let b2 = c2.dbl_iter(&k2.1, e - 3);
    let glue_ok = fin(&a1)
        && fin(&a2)
        && fin(&b1)
        && fin(&b2)
        && !(bool::from(a1.x.ct_eq(&b1.x)) && bool::from(a2.x.ct_eq(&b2.x)));
    let mut report = ChainReport {
        prodker_ok,
        glue_ok,
        first_zero_delta_step: None,
        richelot_steps: 0,
        final_split: false,
    };
    if !glue_ok {
        return report;
    }
    let g = Glue::new(two_torsion_xs(&c1, &a1, &a2), two_torsion_xs(&c2, &b1, &b2));
    let mut gens = [g.map(&k1.0, &k1.1), g.map(&k2.0, &k2.1)];
    let mut h = g.h;
    let order_log = e - 3; // order of gens on the Jacobian
    let r = e - 6;
    for i in 0..r {
        let lv = div_dbl_iter_batch(&h, &gens, order_log - 1 - i);
        let delta = kernel_determinant(&h, &lv[0], &lv[1]);
        if bool::from(delta.is_zero()) {
            report.first_zero_delta_step = Some(i);
            return report;
        }
        let phi = Richelot::new(&h, &lv[0], &lv[1]);
        let m = phi.map_batch(&gens);
        gens = [m[0], m[1]];
        h = phi.hnew;
        report.richelot_steps = i + 1;
    }
    let l0 = div_dbl_iter(&h, &gens[0], 2);
    let l1 = div_dbl_iter(&h, &gens[1], 2);
    report.final_split = bool::from(kernel_determinant(&h, &l0, &l1).is_zero());
    report
}

/// Montgomery model of `e` using a rational 2-torsion x-coordinate `r`.
/// Tries both square roots of the translated linear coefficient so the
/// scaling factor is a square, and keeps the model only when `j` is preserved.
fn try_to_montgomery<F: Field>(e: &WCurve<F>, r: F) -> Option<ToMontgomery<F>> {
    if !bool::from(e.rhs(&r).is_zero()) {
        return None;
    }
    let a2p = e.a2 + r.mul_small(3);
    let a4p = e.a4 + e.a2.double() * r + r.square().mul_small(3);
    let (s, ok) = a4p.sqrt();
    if !bool::from(ok) {
        return None;
    }
    for u2 in [s, -s] {
        let (u, uok) = u2.sqrt();
        if !bool::from(uok) {
            continue;
        }
        let u3 = u2 * u;
        let inv_u2 = u2.invert();
        let inv_u3 = u3.invert();
        if bool::from(inv_u2.is_zero()) || bool::from(inv_u3.is_zero()) {
            continue;
        }
        let tm = ToMontgomery {
            r,
            u2,
            u3,
            inv_u2,
            inv_u3,
            curve: WCurve::from_montgomery(a2p * inv_u2),
        };
        if bool::from(tm.curve.j_invariant().ct_eq(&e.j_invariant())) {
            return Some(tm);
        }
    }
    None
}

/// Put `e` in Montgomery form.  A kernel 2-torsion point used as the origin
/// collapses a keygen kernel, so the two complementary roots are tried first.
fn montgomery_of_curve<F: Field>(e: &WCurve<F>, ker_x: F) -> Option<ToMontgomery<F>> {
    let xs = complementary_2_torsion_xs(e, ker_x);
    for r in [xs[0], xs[1], ker_x] {
        if let Some(tm) = try_to_montgomery(e, r) {
            return Some(tm);
        }
    }
    None
}

/// Key-generation chain in the theta model.
///
/// `k1`, `k2` are a symplectic basis of order `2^{e+2}` on `E × E` whose
/// 4th multiples are the usual order-`2^e` keygen kernel.  [`product_isogeny`]
/// with `n = e` quotients by that kernel (it consumes two bits of 8-torsion
/// for gluing).  `payload` is the five keygen points.
///
/// Kummer y-signs on the split images are left as the principal square
/// roots; the caller recovers the eigenbasis by trying the eight sign
/// patterns against the 2-torsion test.  That is cheaper than pushing
/// extra anchors through the chain and aborting the attempt when they
/// fail to match.
///
/// `None` if the curve cannot be put in Montgomery form.
pub fn keygen_chain_theta<F: Field + Send + Sync>(
    e0: &WCurve<F>,
    k1: &Pair<F>,
    k2: &Pair<F>,
    e: usize,
    payload: &[Pair<F>],
) -> Option<KeygenOut<F>> {
    if e < 2 || payload.len() < 5 {
        return None;
    }
    let t2 = mul_vartime(e0, &k1.0, &(BigUint::from(1u32) << (e + 1)));
    if bool::from(t2.inf) {
        return None;
    }
    let tm = montgomery_of_curve(e0, t2.x)?;
    let map = |p: &WPoint<F>| tm.eval(p);
    let k1 = (map(&k1.0), map(&k1.1));
    let k2 = (map(&k2.0), map(&k2.1));
    let images: Vec<Pair<F>> = payload.iter().map(|p| (map(&p.0), map(&p.1))).collect();
    let strategy = optimal(e, 1.0, 1.0);
    let ((e3, e4), outs) = product_isogeny(&(tm.curve, tm.curve), &k1, &k2, &images, e, &strategy);
    Some((e3, e4, outs))
}

/// Key-generation chain (variable time is acceptable here): schedule
/// glue, rich x (e-2), split, starting from E x E with kernel <K1, K2> of
/// order 2^e.  Returns the two codomain curves and the payload images.
pub fn keygen_chain<F: Field>(
    e0: &WCurve<F>,
    k1: &Pair<F>,
    k2: &Pair<F>,
    e: usize,
    payload: &[Pair<F>],
) -> (WCurve<F>, WCurve<F>, Vec<Pair<F>>) {
    let a1 = e0.dbl_iter(&k1.0, e - 1);
    let a2 = e0.dbl_iter(&k2.0, e - 1);
    let b1 = e0.dbl_iter(&k1.1, e - 1);
    let b2 = e0.dbl_iter(&k2.1, e - 1);
    let g = Glue::new(two_torsion_xs(e0, &a1, &a2), two_torsion_xs(e0, &b1, &b2));
    let mut gens = [g.map(&k1.0, &k1.1), g.map(&k2.0, &k2.1)];
    let mut extra: Vec<Div<F>> = payload.iter().map(|p| g.map(&p.0, &p.1)).collect();
    let h = jacobian_phase(g.h, &mut gens, e - 1, e - 2, &mut extra);
    let s = Split::new(&h, &gens[0], &gens[1]);
    let pay: Vec<Pair<F>> = extra.iter().map(|d| s.map(d)).collect();
    (s.e1, s.e2, pay)
}

/// Does the first (2,2)-step of a keygen kernel glue (no product/graph
/// structure)?  Variable time; used to reject keygen candidates.
pub fn first_step_glues<F: Field>(e0: &WCurve<F>, k1: &Pair<F>, k2: &Pair<F>, e: usize) -> bool {
    let a1 = e0.dbl_iter(&k1.0, e - 1);
    let a2 = e0.dbl_iter(&k2.0, e - 1);
    let b1 = e0.dbl_iter(&k1.1, e - 1);
    let b2 = e0.dbl_iter(&k2.1, e - 1);
    let fin = |p: &WPoint<F>| !bool::from(p.inf);
    // graph of the identity on E[2] (alpha = 1 on E[2]) gives a product step
    fin(&a1)
        && fin(&a2)
        && fin(&b1)
        && fin(&b2)
        && !(bool::from(a1.x.ct_eq(&b1.x)) && bool::from(a2.x.ct_eq(&b2.x)))
}

pub(crate) fn _c(_: Choice) {}
