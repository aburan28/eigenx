//! Level-2 theta (2^n, 2^n)-isogenies between elliptic products.
//!
//! Ported from the Theta-Rust reference implementation by Pierrick Dartois,
//! Luciano Maino, Giacomo Pope and Damien Robert
//! (<https://github.com/ThetaIsogenies/two-isogenies>, MIT license, see
//! `LICENSE.theta-isogenies`): `theta.rs`'s `define_theta_structure!` macro
//! and the elliptic-product helpers of `eccore.rs`.  The underlying
//! algorithms (gluing, intermediate (2,2)-isogeny steps and splitting in the
//! level-2 theta model, with the De Feo–Jao–Plût optimal-strategy
//! traversal) are from "An Algorithmic Approach to (2,2)-isogenies in the
//! Theta Model and Applications to Isogeny-based Cryptography"
//! (Dartois–Maino–Pope–Robert) and <https://eprint.iacr.org/2011/506.pdf>.
//!
//! This port is generic over eigenx's [`Field`] trait rather than the
//! source's per-curve `Fq`/`Curve`/`Point` macros, and reuses `weier.rs`'s
//! general Weierstrass [`WCurve`]/[`WPoint`] rather than introducing a
//! separate Montgomery `Curve`/`Point`/`PointX` type.  Differences worth
//! noting for anyone comparing against the source:
//!
//! - No Montgomery-only x-only (`PointX`/`A24`) arithmetic: `WPoint` always
//!   carries an affine `y` (or an explicit infinity flag), so the
//!   "4-torsion basis-change" submatrix (`get_base_submatrix`) and the
//!   theta <-> elliptic-product bridges are written directly against full
//!   affine coordinates instead of a Kummer line.  This is mathematically
//!   equivalent (the source's `(X : Z)` is just `(x(2T) : 1)` once you
//!   already have `y`) and lets the gluing/splitting code work over *any*
//!   Weierstrass model, not only `y^2 = x^3 + Ax^2 + x`.
//! - [`CouplePoint`] / [`EllipticProduct`] are plain tuples
//!   (`(WPoint<F>, WPoint<F>)`, `(WCurve<F>, WCurve<F>)`) instead of
//!   bespoke structs; see the free functions below for the handful of
//!   tuple-level operations the source hangs off those types.
//! - Field inversions use [`Field::invert`] (0 -> 0, branch-free) one at a
//!   time instead of the source's explicit batch inversion; this is a
//!   performance-only difference (fewer inversions amortised vs. this
//!   port's simplicity), not a correctness one.
//! - The rare 10th splitting-matrix case (reached when
//!   `identify_even_index` returns `(0, 0)`) needs a square root of `-1` in
//!   `F` (the source's `Fq::ZETA`); this port computes it on demand with
//!   `F::sqrt(-1)` instead of requiring a dedicated constant on the
//!   [`Field`] trait. If `-1` is not a square in `F` this branch produces a
//!   wrong (but safe: no panics, no out-of-bounds, no secret branches)
//!   result. It is not expected to be reached by any kernel used in this
//!   crate's KEM (see `tests/theta_chain.rs`).
//!
//! Variable time: following the source, the 4-torsion basis-change search
//! in [`get_base_matrix`]/[`get_base_submatrix`] (run once, on public or
//! already-validated points, at the start of [`product_isogeny`]) is the
//! only place with an unguarded field inversion of a quantity that could
//! be zero for pathological input (`det = x - u`); for the maximal
//! isotropic kernels this module is used with, `det` is non-zero. All
//! per-point formulas (gluing/isogeny/splitting codomain and image
//! computations) use masked, branch-free selection (`chi_eval`,
//! `zero_index`, `identify_even_index`) exactly as in the source, so they
//! carry no secret-dependent control flow even though they are evaluated
//! on secret kernel data in `decaps_chain_theta`.

use crate::ct::{mask, Choice, ConditionallySelectable};
use crate::field::Field;
use crate::weier::{WCurve, WPoint};

/// A point of `E1 x E2`: `eccore.rs`'s `CouplePoint`.
pub type CouplePoint<F> = (WPoint<F>, WPoint<F>);

/// A product of two elliptic curves `E1 x E2`: `eccore.rs`'s
/// `EllipticProduct`.
pub type EllipticProduct<F> = (WCurve<F>, WCurve<F>);

/// Invert `n` field elements with one inversion. Zeros invert to zero.
/// Constant time in the values (the length `n` is public).
fn batch_invert<F: Field>(v: &mut [F]) {
    let n = v.len();
    if n == 0 {
        return;
    }
    let mut nz = vec![F::one(); n];
    let mut zero = vec![Choice::from(0u8); n];
    for i in 0..n {
        zero[i] = v[i].is_zero();
        nz[i] = F::conditional_select(&v[i], &F::one(), zero[i]);
    }
    let mut prefix = vec![F::one(); n];
    prefix[0] = nz[0];
    for i in 1..n {
        prefix[i] = prefix[i - 1] * nz[i];
    }
    let mut inv = prefix[n - 1].invert();
    for i in (1..n).rev() {
        let ii = inv * prefix[i - 1];
        inv *= nz[i];
        v[i] = F::conditional_select(&ii, &F::zero(), zero[i]);
    }
    v[0] = F::conditional_select(&inv, &F::zero(), zero[0]);
}

#[inline(always)]
fn ep_add<F: Field>(
    e: &EllipticProduct<F>,
    a: &CouplePoint<F>,
    b: &CouplePoint<F>,
) -> CouplePoint<F> {
    (e.0.add(&a.0, &b.0), e.1.add(&a.1, &b.1))
}

fn curve_is_mont<F: Field>(e: &WCurve<F>) -> bool {
    bool::from(e.a6.is_zero()) && bool::from(e.a4.ct_eq(&F::one()))
}

fn one_dbl<F: Field>(e: &WCurve<F>, p: &WPoint<F>, n: usize, mont: bool) -> WPoint<F> {
    if mont {
        e.dbl_iter_mont(p, n)
    } else {
        e.dbl_iter(p, n)
    }
}

#[inline(always)]
fn ep_double_iter<F: Field>(
    e: &EllipticProduct<F>,
    a: &CouplePoint<F>,
    n: usize,
) -> CouplePoint<F> {
    let mont = curve_is_mont(&e.0) && curve_is_mont(&e.1);
    (one_dbl(&e.0, &a.0, n, mont), one_dbl(&e.1, &a.1, n, mont))
}

/// Double two couple points on the elliptic product.
fn ep_double_pair<F: Field>(
    e: &EllipticProduct<F>,
    a: &CouplePoint<F>,
    b: &CouplePoint<F>,
    n: usize,
) -> (CouplePoint<F>, CouplePoint<F>) {
    (ep_double_iter(e, a, n), ep_double_iter(e, b, n))
}

#[inline(always)]
fn mask64(c: Choice) -> u64 {
    mask(c)
}

/// Hadamard transform of four field elements.  Cost: 8a.
#[inline(always)]
fn to_hadamard<F: Field>(x: F, y: F, z: F, t: F) -> (F, F, F, F) {
    let t1 = x + y;
    let t2 = x - y;
    let t3 = z + t;
    let t4 = z - t;
    (t1 + t3, t2 + t4, t1 - t3, t2 - t4)
}

/// Square each of four field elements.  Cost: 4S.
#[inline(always)]
fn to_squared_coords<F: Field>(x: F, y: F, z: F, t: F) -> (F, F, F, F) {
    (x.square(), y.square(), z.square(), t.square())
}

/// Square each coordinate, then Hadamard transform.  Cost: 4S + 8a.
#[inline(always)]
fn to_squared_theta<F: Field>(x: F, y: F, z: F, t: F) -> (F, F, F, F) {
    let (xx, yy, zz, tt) = to_squared_coords(x, y, z, t);
    to_hadamard(xx, yy, zz, tt)
}

// ============================================================================
// ThetaPoint / ThetaStructure
// ============================================================================

/// An element of the level-2 theta structure: four projective coordinates
/// `(x : y : z : t)`.
#[derive(Clone, Copy, Debug)]
pub struct ThetaPoint<F: Field> {
    pub x: F,
    pub y: F,
    pub z: F,
    pub t: F,
}

impl<F: Field> ThetaPoint<F> {
    pub fn from_coords(x: F, y: F, z: F, t: F) -> Self {
        ThetaPoint { x, y, z, t }
    }
    pub fn coords(&self) -> (F, F, F, F) {
        (self.x, self.y, self.z, self.t)
    }
    pub fn list(&self) -> [F; 4] {
        [self.x, self.y, self.z, self.t]
    }
    pub fn hadamard(&self) -> (F, F, F, F) {
        to_hadamard(self.x, self.y, self.z, self.t)
    }
    #[inline]
    pub fn squared_theta(&self) -> (F, F, F, F) {
        to_squared_theta(self.x, self.y, self.z, self.t)
    }
}

/// The parent of [`ThetaPoint`]: the null point of the level-2 theta
/// structure plus an 8-element arithmetic precomputation reused for
/// doubling and isogeny evaluation.
#[derive(Clone, Copy, Debug)]
pub struct ThetaStructure<F: Field> {
    null_point: ThetaPoint<F>,
    precom: [F; 8],
}

impl<F: Field> ThetaStructure<F> {
    pub fn new_from_coords(x: F, y: F, z: F, t: F) -> Self {
        let null_point = ThetaPoint::from_coords(x, y, z, t);
        ThetaStructure {
            null_point,
            precom: Self::precomputation(&null_point),
        }
    }
    pub fn new_from_point(null_point: &ThetaPoint<F>) -> Self {
        ThetaStructure {
            null_point: *null_point,
            precom: Self::precomputation(null_point),
        }
    }
    pub fn null_point(&self) -> ThetaPoint<F> {
        self.null_point
    }

    /// 8 constants needed for doubling and isogeny computations.  Cost:
    /// 14M + 5S.
    pub fn precomputation(o0: &ThetaPoint<F>) -> [F; 8] {
        let (a, b, c, d) = o0.coords();
        let (aa, bb, cc, dd) = to_squared_coords(a, b, c, d);

        let ab = a * b;
        let cd = c * d;
        let x0 = ab * cd;
        let y0 = aa * cd;
        let aab = aa * b;
        let z0 = aab * d;
        let t0 = aab * c;

        let (haa, hbb, hcc, hdd) = to_hadamard(aa, bb, cc, dd);
        let a4 = haa.square();
        let aabb = haa * hbb;
        let ccdd = hcc * hdd;
        let xx0 = aabb * ccdd;
        let yy0 = a4 * ccdd;
        let a4bb = a4 * hbb;
        let zz0 = a4bb * hdd;
        let tt0 = a4bb * hcc;

        [x0, y0, z0, t0, xx0, yy0, zz0, tt0]
    }

    /// `P <- [2]P` in place.  Cost: 8S + 8M.
    #[inline(always)]
    pub fn set_double_self(&self, p: &mut ThetaPoint<F>) {
        let (mut xp, mut yp, mut zp, mut tp) = p.squared_theta();
        xp = self.precom[4] * xp.square();
        yp = self.precom[5] * yp.square();
        zp = self.precom[6] * zp.square();
        tp = self.precom[7] * tp.square();

        let (mut x, mut y, mut z, mut t) = to_hadamard(xp, yp, zp, tp);
        x *= self.precom[0];
        y *= self.precom[1];
        z *= self.precom[2];
        t *= self.precom[3];
        p.x = x;
        p.y = y;
        p.z = z;
        p.t = t;
    }

    /// Double two independent points together so their field products overlap.
    #[inline(always)]
    fn set_double_pair(&self, p: &mut ThetaPoint<F>, q: &mut ThetaPoint<F>) {
        let (xp, yp, zp, tp) = p.squared_theta();
        let (xq, yq, zq, tq) = q.squared_theta();
        let (c0, c1, c2, c3) = (
            self.precom[0],
            self.precom[1],
            self.precom[2],
            self.precom[3],
        );
        let (c4, c5, c6, c7) = (
            self.precom[4],
            self.precom[5],
            self.precom[6],
            self.precom[7],
        );
        let xp = c4 * xp.square();
        let xq = c4 * xq.square();
        let yp = c5 * yp.square();
        let yq = c5 * yq.square();
        let zp = c6 * zp.square();
        let zq = c6 * zq.square();
        let tp = c7 * tp.square();
        let tq = c7 * tq.square();
        let (xp, yp, zp, tp) = to_hadamard(xp, yp, zp, tp);
        let (xq, yq, zq, tq) = to_hadamard(xq, yq, zq, tq);
        p.x = c0 * xp;
        q.x = c0 * xq;
        p.y = c1 * yp;
        q.y = c1 * yq;
        p.z = c2 * zp;
        q.z = c2 * zq;
        p.t = c3 * tp;
        q.t = c3 * tq;
    }

    pub fn double_point(&self, p: &ThetaPoint<F>) -> ThetaPoint<F> {
        let mut p2 = *p;
        self.set_double_self(&mut p2);
        p2
    }

    pub fn double_iter(&self, p: &ThetaPoint<F>, n: usize) -> ThetaPoint<F> {
        let mut r = *p;
        for _ in 0..n {
            self.set_double_self(&mut r);
        }
        r
    }

    #[inline(always)]
    fn double_iter_pair(
        &self,
        p: &ThetaPoint<F>,
        q: &ThetaPoint<F>,
        n: usize,
    ) -> (ThetaPoint<F>, ThetaPoint<F>) {
        let mut p = *p;
        let mut q = *q;
        for _ in 0..n {
            self.set_double_pair(&mut p, &mut q);
        }
        (p, q)
    }
}

// ============================================================================
// Gluing: elliptic product -> level-2 theta structure
// ============================================================================

/// Given a point `T` of exact order 4 on `E`, the 2x2 matrix needed for the
/// gluing basis change, `M = [[a, b], [c, d]]`.
///
/// The source computes this from the `(X : Z)` Kummer coordinates of `T`
/// and `[2]T` via a Montgomery-only x-only doubling formula.  Since
/// eigenx's `WPoint` always carries a genuine affine `y`, `T` and `[2]T`
/// are both already affine with `Z = 1` (both are finite: `T` has order 4,
/// `[2]T` has order 2), which lets us skip the x-only formula and the
/// resulting batch inversion entirely; the two forms agree algebraically.
///
/// Variable time: the single inversion below is of `x(T) - x([2]T)`, which
/// is the standard genericity condition for this basis change (it is the
/// same condition the source's batched inversion silently assumes).
fn get_base_submatrix<F: Field>(e: &WCurve<F>, t: &WPoint<F>) -> (F, F, F, F) {
    let x = t.x;
    let t2 = e.add(t, t);
    let u = t2.x;
    let det_inv = (x - u).invert();
    let d = u * det_inv;
    let a = -d;
    let b = -det_inv;
    let c = u * x * det_inv - x;
    (a, b, c, d)
}

/// Given the 4-torsion below the (2,2)-isogeny kernel, the symplectic
/// basis-change matrix used to express the kernel in a gluing-compatible
/// form.  `p1p2`, `q1q2` are a symplectic basis of `(E1 x E2)[4]` above the
/// kernel.
fn get_base_matrix<F: Field>(
    e1e2: &EllipticProduct<F>,
    p1p2: &CouplePoint<F>,
    q1q2: &CouplePoint<F>,
) -> [F; 16] {
    let (e1, e2) = *e1e2;
    let (p1, p2) = *p1p2;
    let (q1, q2) = *q1q2;

    let (g00_1, g01_1, g10_1, g11_1) = get_base_submatrix(&e1, &p1);
    let (g00_2, g01_2, g10_2, g11_2) = get_base_submatrix(&e2, &p2);
    let (h00_1, _h01_1, h10_1, _h11_1) = get_base_submatrix(&e1, &q1);
    let (h00_2, h01_2, h10_2, h11_2) = get_base_submatrix(&e2, &q2);

    let gh00_1 = g00_1 * h00_1 + g01_1 * h10_1;
    let gh10_1 = g10_1 * h00_1 + g11_1 * h10_1;
    let gh00_2 = g00_2 * h00_2 + g01_2 * h10_2;
    let gh10_2 = g10_2 * h00_2 + g11_2 * h10_2;

    let mut a = F::one();
    let mut b = F::zero();
    let mut c = F::zero();
    let mut d = F::zero();

    a += g00_1 * g00_2;
    b += g00_1 * g10_2;
    c += g10_1 * g00_2;
    d += g10_1 * g10_2;

    a += h00_1 * h00_2;
    b += h00_1 * h10_2;
    c += h10_1 * h00_2;
    d += h10_1 * h10_2;

    a += gh00_1 * gh00_2;
    b += gh00_1 * gh10_2;
    c += gh10_1 * gh00_2;
    d += gh10_1 * gh10_2;

    let a1 = h00_2 * a + h01_2 * b;
    let b1 = h10_2 * a + h11_2 * b;
    let c1 = h00_2 * c + h01_2 * d;
    let d1 = h10_2 * c + h11_2 * d;

    let a2 = g00_1 * a + g01_1 * c;
    let b2 = g00_1 * b + g01_1 * d;
    let c2 = g10_1 * a + g11_1 * c;
    let d2 = g10_1 * b + g11_1 * d;

    let a3 = g00_1 * a1 + g01_1 * c1;
    let b3 = g00_1 * b1 + g01_1 * d1;
    let c3 = g10_1 * a1 + g11_1 * c1;
    let d3 = g10_1 * b1 + g11_1 * d1;

    [a, b, c, d, a1, b1, c1, d1, a2, b2, c2, d2, a3, b3, c3, d3]
}

/// Apply the base change described by `m` on a theta point, in place.
/// Cost: 16M.
fn apply_base_change<F: Field>(p: &mut ThetaPoint<F>, m: [F; 16]) {
    let (x, y, z, t) = p.coords();
    p.x = m[0] * x + m[1] * y + m[2] * z + m[3] * t;
    p.y = m[4] * x + m[5] * y + m[6] * z + m[7] * t;
    p.z = m[8] * x + m[9] * y + m[10] * z + m[11] * t;
    p.t = m[12] * x + m[13] * y + m[14] * z + m[15] * t;
}

/// Map a couple point to a level-2 theta point and apply the gluing basis
/// change `m`.  Cost: 20M.
fn base_change_couple_point<F: Field>(p1p2: &CouplePoint<F>, m: [F; 16]) -> ThetaPoint<F> {
    let (p1, p2) = *p1p2;
    // (X : Z) Kummer coordinates, swapping the point at infinity (0 : 0)
    // for (1 : 0), branch-free, as in the source.
    let x1 = F::conditional_select(&p1.x, &F::one(), p1.inf);
    let z1 = F::conditional_select(&F::one(), &F::zero(), p1.inf);
    let x2 = F::conditional_select(&p2.x, &F::one(), p2.inf);
    let z2 = F::conditional_select(&F::one(), &F::zero(), p2.inf);

    let x = x1 * x2;
    let y = x1 * z2;
    let z = z1 * x2;
    let t = z1 * z2;
    let mut p = ThetaPoint::from_coords(x, y, z, t);
    apply_base_change(&mut p, m);
    p
}

/// Find the (unique, for valid input) zero entry of a 4-element dual
/// coordinate vector, branch-free.
fn zero_index<F: Field>(d: &[F; 4]) -> usize {
    let mut idx: u64 = 0;
    for (i, el) in d.iter().enumerate() {
        idx |= (i as u64) & mask64(el.is_zero());
    }
    idx as usize
}

/// Given the 8-torsion above the kernel of order 2, the codomain
/// `ThetaStructure` of the gluing (2,2)-isogeny, plus the two inverses and
/// the zero index needed to evaluate images.  Cost: 8S + 13M + 1I.
fn gluing_codomain<F: Field>(
    t1_8: &ThetaPoint<F>,
    t2_8: &ThetaPoint<F>,
) -> (ThetaStructure<F>, (F, F), usize) {
    let (xa, xb, xc, xd) = t1_8.squared_theta();
    let xaxbycyd = [xa, xb, xc, xd];
    let (za, zb, zc, zd) = t2_8.squared_theta();
    let zatbzytd = [za, zb, zc, zd];

    let z_idx = zero_index(&xaxbycyd);

    let t1v = zatbzytd[1 ^ z_idx];
    let t2v = xaxbycyd[2 ^ z_idx];
    let t3v = zatbzytd[3 ^ z_idx];
    let t4v = xaxbycyd[3 ^ z_idx];

    let mut invs = [t1v, t2v, t3v, t4v];
    batch_invert(&mut invs);
    let [i1, i2, i3, i4] = invs;

    let mut abcd = [F::zero(); 4];
    // XOR-index pattern matches the source (0^z_idx ≡ z_idx).
    #[allow(clippy::identity_op)]
    {
        abcd[0 ^ z_idx] = F::zero();
        abcd[1 ^ z_idx] = t1v * i3;
        abcd[2 ^ z_idx] = t2v * i4;
        abcd[3 ^ z_idx] = F::one();
    }

    let a_inv = t3v * i1;
    let b_inv = t4v * i2;

    let (a, b, c, d) = to_hadamard(abcd[0], abcd[1], abcd[2], abcd[3]);
    let codomain = ThetaStructure::new_from_coords(a, b, c, d);

    (codomain, (a_inv, b_inv), z_idx)
}

/// Push a point `t` (with shifted value `t_shift`) through the gluing
/// isogeny, given the constants precomputed by [`gluing_codomain`].  Cost:
/// 8S + 10M + 2I.
fn gluing_image<F: Field>(
    t: &ThetaPoint<F>,
    t_shift: &ThetaPoint<F>,
    a_inv: F,
    b_inv: F,
    z_idx: usize,
) -> ThetaPoint<F> {
    let (xa, xb, xc, xd) = t.squared_theta();
    let axbyczdt = [xa, xb, xc, xd];
    let (ya, yb, yc, yd) = t_shift.squared_theta();
    let aybxctdz = [ya, yb, yc, yd];

    let y = axbyczdt[1 ^ z_idx] * a_inv;
    let z = axbyczdt[2 ^ z_idx] * b_inv;
    let tt = axbyczdt[3 ^ z_idx];

    let zb = aybxctdz[3 ^ z_idx];
    let tb = aybxctdz[2 ^ z_idx] * b_inv;

    // Both inversions are computed unconditionally (inverting zero gives
    // zero) and the right one selected branch-free, matching the source.
    let mut invs = [zb, tb];
    batch_invert(&mut invs);
    let [izb, itb] = invs;
    let lam = F::conditional_select(&(z * izb), &(tt * itb), z.is_zero());

    let xb = aybxctdz[1 ^ z_idx] * a_inv;
    let x = xb * lam;

    let mut xyzt = [F::zero(); 4];
    #[allow(clippy::identity_op)]
    {
        xyzt[0 ^ z_idx] = x;
        xyzt[1 ^ z_idx] = y;
        xyzt[2 ^ z_idx] = z;
        xyzt[3 ^ z_idx] = tt;
    }

    let (xo, yo, zo, to) = to_hadamard(xyzt[0], xyzt[1], xyzt[2], xyzt[3]);
    ThetaPoint::from_coords(xo, yo, zo, to)
}

/// Compute the gluing (2,2)-isogeny from an elliptic-product
/// `ThetaStructure`.  `p1p2_8`, `q1q2_8` are the 8-torsion above the
/// kernel of order 2.
fn gluing_isogeny<F: Field>(
    e1e2: &EllipticProduct<F>,
    p1p2_8: &CouplePoint<F>,
    q1q2_8: &CouplePoint<F>,
    image_points: &[CouplePoint<F>],
) -> (ThetaStructure<F>, Vec<ThetaPoint<F>>) {
    let p1p2_4 = ep_double_iter(e1e2, p1p2_8, 1);
    let q1q2_4 = ep_double_iter(e1e2, q1q2_8, 1);

    let m = get_base_matrix(e1e2, &p1p2_4, &q1q2_4);

    let t1_8 = base_change_couple_point(p1p2_8, m);
    let t2_8 = base_change_couple_point(q1q2_8, m);

    let (codomain, (a_inv, b_inv), z_idx) = gluing_codomain(&t1_8, &t2_8);

    let mut theta_images: Vec<ThetaPoint<F>> = Vec::with_capacity(image_points.len());
    for p in image_points.iter() {
        let p_sum_t = ep_add(e1e2, p, &p1p2_4);
        let t = base_change_couple_point(p, m);
        let t_shift = base_change_couple_point(&p_sum_t, m);
        theta_images.push(gluing_image(&t, &t_shift, a_inv, b_inv, z_idx));
    }

    (codomain, theta_images)
}

// ============================================================================
// General (2,2)-isogenies between theta structures
// ============================================================================

/// Given the 8-torsion above the kernel, the codomain of the (2,2)-isogeny
/// and the images of `image_points`.  `hadamard` controls whether a
/// symplectic transform is needed to avoid zero coordinates near a product
/// structure (see the source's module comment).
/// Cost: codomain 8S + 9M, image (per point) 4S + 4M.
fn two_isogeny<F: Field + Send + Sync>(
    t1: &ThetaPoint<F>,
    t2: &ThetaPoint<F>,
    image_points: &mut [ThetaPoint<F>],
    hadamard: [bool; 2],
) -> ThetaStructure<F> {
    let (xa, xb, _, _) = t1.squared_theta();
    let (za, tb, zc, td) = t2.squared_theta();

    let xatb = xa * tb;
    let zaxb = za * xb;
    let zctd = zc * td;

    let mut a = za * xatb;
    let mut b = tb * zaxb;
    let mut c = zc * xatb;
    let mut d = td * zaxb;

    let a_inv = xb * zctd;
    let b_inv = xa * zctd;
    let c_inv = d;
    let d_inv = c;

    if hadamard[1] {
        let (ha, hb, hc, hd) = to_hadamard(a, b, c, d);
        a = ha;
        b = hb;
        c = hc;
        d = hd;
    }
    let codomain = ThetaStructure::new_from_coords(a, b, c, d);
    map_isog_points(image_points, a_inv, b_inv, c_inv, d_inv, hadamard);
    codomain
}

fn eval_isog_point<F: Field>(
    p: &mut ThetaPoint<F>,
    a_inv: F,
    b_inv: F,
    c_inv: F,
    d_inv: F,
    hadamard: [bool; 2],
) {
    let (mut xx, mut yy, mut zz, mut tt) = p.coords();
    if hadamard[0] {
        let (hx, hy, hz, ht) = to_hadamard(xx, yy, zz, tt);
        let (sx, sy, sz, st) = to_squared_theta(hx, hy, hz, ht);
        xx = sx;
        yy = sy;
        zz = sz;
        tt = st;
    } else {
        let (sx, sy, sz, st) = to_squared_theta(xx, yy, zz, tt);
        xx = sx;
        yy = sy;
        zz = sz;
        tt = st;
    }
    xx *= a_inv;
    yy *= b_inv;
    zz *= c_inv;
    tt *= d_inv;
    if hadamard[1] {
        let (hx, hy, hz, ht) = to_hadamard(xx, yy, zz, tt);
        xx = hx;
        yy = hy;
        zz = hz;
        tt = ht;
    }
    p.x = xx;
    p.y = yy;
    p.z = zz;
    p.t = tt;
}

/// Hot path of an intermediate (2,2)-step: no input Hadamard, output Hadamard.
#[inline(always)]
fn eval_isog_out_hadamard<F: Field>(p: &mut ThetaPoint<F>, a_inv: F, b_inv: F, c_inv: F, d_inv: F) {
    let (xx, yy, zz, tt) = to_squared_theta(p.x, p.y, p.z, p.t);
    let (hx, hy, hz, ht) = to_hadamard(xx * a_inv, yy * b_inv, zz * c_inv, tt * d_inv);
    p.x = hx;
    p.y = hy;
    p.z = hz;
    p.t = ht;
}

#[inline(always)]
fn eval_isog_out_hadamard_pair<F: Field>(
    p: &mut ThetaPoint<F>,
    q: &mut ThetaPoint<F>,
    a_inv: F,
    b_inv: F,
    c_inv: F,
    d_inv: F,
) {
    let (px, py, pz, pt) = to_squared_theta(p.x, p.y, p.z, p.t);
    let (qx, qy, qz, qt) = to_squared_theta(q.x, q.y, q.z, q.t);
    let (px, py, pz, pt) = to_hadamard(px * a_inv, py * b_inv, pz * c_inv, pt * d_inv);
    let (qx, qy, qz, qt) = to_hadamard(qx * a_inv, qy * b_inv, qz * c_inv, qt * d_inv);
    p.x = px;
    p.y = py;
    p.z = pz;
    p.t = pt;
    q.x = qx;
    q.y = qy;
    q.z = qz;
    q.t = qt;
}

fn map_isog_points<F: Field>(
    image_points: &mut [ThetaPoint<F>],
    a_inv: F,
    b_inv: F,
    c_inv: F,
    d_inv: F,
    hadamard: [bool; 2],
) {
    if hadamard == [false, true] {
        let mut i = 0;
        while i + 1 < image_points.len() {
            let (left, right) = image_points.split_at_mut(i + 1);
            eval_isog_out_hadamard_pair(&mut left[i], &mut right[0], a_inv, b_inv, c_inv, d_inv);
            i += 2;
        }
        if i < image_points.len() {
            eval_isog_out_hadamard(&mut image_points[i], a_inv, b_inv, c_inv, d_inv);
        }
        return;
    }
    for p in image_points.iter_mut() {
        eval_isog_point(p, a_inv, b_inv, c_inv, d_inv, hadamard);
    }
}

/// Special-cased (2,2)-isogeny for the step landing on a product of
/// elliptic curves (one dual coordinate is zero).  Cost: codomain 8S +
/// 13M, image (per point) 4S + 3M.
fn two_isogeny_to_product<F: Field>(
    t1: &ThetaPoint<F>,
    t2: &ThetaPoint<F>,
    image_points: &mut [ThetaPoint<F>],
) -> ThetaStructure<F> {
    let (xa0, xb0, yc0, yd0) = t1.hadamard();
    let (xa, xb, _, _) = to_squared_theta(xa0, xb0, yc0, yd0);

    let (za0, tb0, zc0, td0) = t2.hadamard();
    let (za, tb, zc, td) = to_squared_theta(za0, tb0, zc0, td0);

    let zatb = za * tb;
    let a = xa * zatb;
    let b = xb * zatb;
    let c = zc * xa * tb;
    let d = td * xb * za;

    let ab = a * b;
    let cd = c * d;
    let a_inv = cd * b;
    let b_inv = cd * a;
    let c_inv = ab * d;
    let d_inv = ab * c;

    let codomain = ThetaStructure::new_from_coords(a, b, c, d);

    for p in image_points.iter_mut() {
        let (xx, yy, zz, tt) = p.coords();
        let (hx, hy, hz, ht) = to_hadamard(xx, yy, zz, tt);
        let (sx, sy, sz, st) = to_squared_theta(hx, hy, hz, ht);
        p.x = sx * a_inv;
        p.y = sy * b_inv;
        p.z = sz * c_inv;
        p.t = st * d_inv;
    }

    codomain
}

// ============================================================================
// Splitting: symplectic transform + map to a product of elliptic curves
// ============================================================================

/// `(-1)^{i . j}` as a branch-free mask (`u64::MAX` for `-1`, `0` for `1`),
/// where `i, j` in `{0,..,3}` encode pairs `(x, y)` in `{0,1}^2`.  See the
/// source's module comment for the derivation.
fn chi_eval(x: usize, y: usize) -> u64 {
    match (x, y) {
        (1, 1) | (1, 3) | (2, 2) | (2, 3) | (3, 1) | (3, 2) => u64::MAX,
        _ => 0,
    }
}

/// For a given index `(chi, i)`, the level-(2,2) constant (squared); zero
/// exactly at the even index identifying the current symplectic position.
fn level_22_constants_sqr<F: Field>(null_point: &ThetaPoint<F>, chi: usize, i: usize) -> F {
    let mut u = F::zero();
    let coords = null_point.list();
    for t in 0..4 {
        let u_it = coords[t] * coords[i ^ t];
        let neg = Choice::from((chi_eval(chi, t) & 1) as u8);
        u += F::conditional_select(&u_it, &(-u_it), neg);
    }
    u
}

/// The (unique, for valid input) even index `(chi, i)` whose level-(2,2)
/// constant vanishes, found branch-free by evaluating all 10 candidates.
fn identify_even_index<F: Field>(null_point: &ThetaPoint<F>) -> (usize, usize) {
    const EVEN_INDICES: [(usize, usize); 10] = [
        (0, 0),
        (0, 1),
        (0, 2),
        (0, 3),
        (1, 0),
        (1, 2),
        (2, 0),
        (2, 1),
        (3, 0),
        (3, 3),
    ];
    let mut chi_zero: u64 = 0;
    let mut i_zero: u64 = 0;
    for (chi, i) in EVEN_INDICES.iter() {
        let u_sqr = level_22_constants_sqr(null_point, *chi, *i);
        let m = mask64(u_sqr.is_zero());
        chi_zero |= (*chi as u64) & m;
        i_zero |= (*i as u64) & m;
    }
    (chi_zero as usize, i_zero as usize)
}

/// The symplectic transform mapping a theta null point into the form that
/// admits a splitting to a product of elliptic curves: one of 10
/// precomputed matrices, selected by [`identify_even_index`].
///
/// The 10th case (`identify_even_index` returns `(0, 0)`) needs a square
/// root of `-1`; see the module-level doc comment.
fn compute_splitting_matrix<F: Field>(null_point: &ThetaPoint<F>) -> [F; 16] {
    let one = F::one();
    let zero = F::zero();
    let mone = -one;
    #[rustfmt::skip]
    let maps: [[F; 16]; 10] = [
        [
            one, zero, zero, zero,
            zero, one, zero, zero,
            zero, zero, zero, one,
            zero, zero, mone, zero,
        ],
        [
            one, zero, zero, zero,
            zero, one, zero, zero,
            zero, zero, one, zero,
            zero, zero, zero, one,
        ],
        [
            one, zero, zero, zero,
            zero, one, zero, zero,
            zero, zero, one, zero,
            zero, zero, zero, mone,
        ],
        [
            one, one, one, one,
            one, mone, one, mone,
            one, mone, mone, one,
            one, one, mone, mone,
        ],
        [
            one, zero, zero, zero,
            zero, zero, zero, one,
            zero, zero, one, zero,
            zero, mone, zero, zero,
        ],
        [
            one, zero, zero, zero,
            zero, one, zero, zero,
            zero, zero, zero, one,
            zero, zero, one, zero,
        ],
        [
            one, one, one, one,
            one, mone, one, mone,
            one, mone, mone, one,
            mone, mone, one, one,
        ],
        [
            one, one, one, one,
            one, mone, one, mone,
            one, one, mone, mone,
            mone, one, one, mone,
        ],
        [
            one, one, one, one,
            one, mone, mone, one,
            one, one, mone, mone,
            mone, one, mone, one,
        ],
        {
            let zeta = F::sqrt(&mone).0;
            let mzeta = -zeta;
            [
                one, zeta, one, zeta,
                one, mzeta, one, zeta,
                one, zeta, one, mzeta,
                one, zeta, one, zeta,
            ]
        },
    ];

    match identify_even_index(null_point) {
        (0, 2) => maps[0],
        (3, 3) => maps[1],
        (0, 3) => maps[2],
        (2, 1) => maps[3],
        (0, 1) => maps[4],
        (1, 2) => maps[5],
        (2, 0) => maps[6],
        (3, 0) => maps[7],
        (1, 0) => maps[8],
        // (0, 0), and (unreachable for valid input) anything else.
        _ => maps[9],
    }
}

/// Map a `ThetaStructure` (and its tracked points) through the splitting
/// symplectic transform.
fn splitting_isomorphism<F: Field>(
    th: ThetaStructure<F>,
    image_points: &mut [ThetaPoint<F>],
) -> ThetaStructure<F> {
    let mut o0 = th.null_point();
    let m = compute_splitting_matrix(&o0);
    apply_base_change(&mut o0, m);
    for p in image_points.iter_mut() {
        apply_base_change(p, m);
    }
    ThetaStructure::new_from_point(&o0)
}

/// Split a theta point into two dimension-1 theta points `(a, b)`, `(b,
/// d)`.  Algorithm from "Models of Kummer lines and Galois representation"
/// (Barbulescu–Robert–Sarkis).
fn split_theta_point<F: Field>(p: &ThetaPoint<F>) -> ((F, F), (F, F)) {
    let (a, b, _, d) = p.coords();
    ((a, b), (b, d))
}

/// Recover the Montgomery coefficient of the elliptic curve corresponding
/// to a dimension-1 null theta point.
fn null_point_to_montgomery_curve<F: Field>(o0: &(F, F)) -> WCurve<F> {
    let (a, b) = *o0;
    let aa = a.square();
    let bb = b.square();
    let t1 = aa + bb;
    let t2 = aa - bb;
    let big_a = -(t1.square() + t2.square()) * (t1 * t2).invert();
    WCurve::from_montgomery(big_a)
}

/// Recover a point on `e` (the elliptic curve of null point `o0`) from a
/// dimension-1 theta point `p`.  Returns the point at infinity if the
/// Kummer `(X : Z)` representation has `Z = 0`, or (should not happen for
/// honest chain data) `rhs(x)` is not a square.
fn theta_point_to_weier_point<F: Field>(e: &WCurve<F>, o0: &(F, F), p: &(F, F)) -> WPoint<F> {
    let (a, b) = *o0;
    let (u, v) = *p;
    let num = a * v + b * u;
    let den = a * v - b * u;
    let den_zero = den.is_zero();
    let x = num * den.invert();
    let (y, ok) = e.rhs(&x).sqrt();
    let r = WPoint {
        x,
        y,
        inf: Choice::from(0),
    };
    WPoint::conditional_select(&r, &WPoint::infinity(), den_zero | !ok)
}

/// Given a `ThetaStructure` and a set of points in the compatible
/// (post-[`splitting_isomorphism`]) form, split to a product of elliptic
/// curves and the corresponding points.
fn split_to_product<F: Field>(
    th: &ThetaStructure<F>,
    image_points: &[ThetaPoint<F>],
    num_image_points: usize,
) -> (EllipticProduct<F>, Vec<CouplePoint<F>>) {
    let null_point = th.null_point();
    let (o1, o2) = split_theta_point(&null_point);

    let e3 = null_point_to_montgomery_curve(&o1);
    let e4 = null_point_to_montgomery_curve(&o2);

    let mut couple_points = Vec::with_capacity(num_image_points);
    for p in image_points.iter().take(num_image_points) {
        let (p1, p2) = split_theta_point(p);
        let q1 = theta_point_to_weier_point(&e3, &o1, &p1);
        let q2 = theta_point_to_weier_point(&e4, &o2, &p2);
        couple_points.push((q1, q2));
    }

    ((e3, e4), couple_points)
}

// ============================================================================
// Main entry points: isogenies between elliptic products
// ============================================================================

/// Compute a (2^n, 2^n)-isogeny between elliptic products with kernel
/// lifts `<p1p2, q1q2>` of order `2^{n+2}` (the extra two bits supply the
/// 8-torsion consumed by gluing; the isogeny degree is `2^n`), naive
/// method with no optimised strategy.  Quadratic in `n`; for tests and
/// small `n` only (see [`product_isogeny`] for the strategy-driven version
/// used in `decaps_chain_theta`).
pub fn product_isogeny_no_strategy<F: Field + Send + Sync>(
    e1e2: &EllipticProduct<F>,
    p1p2: &CouplePoint<F>,
    q1q2: &CouplePoint<F>,
    image_points: &[CouplePoint<F>],
    n: usize,
) -> (EllipticProduct<F>, Vec<CouplePoint<F>>) {
    let num_image_points = image_points.len();
    let mut kernel_couple_pts: Vec<CouplePoint<F>> = image_points.to_vec();
    kernel_couple_pts.push(*p1p2);
    kernel_couple_pts.push(*q1q2);

    let p1p2_8 = ep_double_iter(e1e2, p1p2, n - 1);
    let q1q2_8 = ep_double_iter(e1e2, q1q2, n - 1);

    let (mut domain, mut kernel_pts) = gluing_isogeny(e1e2, &p1p2_8, &q1q2_8, &kernel_couple_pts);

    for k in 1..n {
        let tp1 = domain.double_iter(&kernel_pts[num_image_points], n - k - 1);
        let tp2 = domain.double_iter(&kernel_pts[num_image_points + 1], n - k - 1);
        if k == n - 2 {
            domain = two_isogeny(&tp1, &tp2, &mut kernel_pts, [false, false]);
        } else if k == n - 1 {
            domain = two_isogeny_to_product(&tp1, &tp2, &mut kernel_pts);
        } else {
            domain = two_isogeny(&tp1, &tp2, &mut kernel_pts, [false, true]);
        }
    }

    domain = splitting_isomorphism(domain, &mut kernel_pts);
    split_to_product(&domain, &kernel_pts, num_image_points)
}

/// Compute a (2^n, 2^n)-isogeny between elliptic products with kernel
/// lifts `<p1p2, q1q2>` of order `2^{n+2}` (degree `2^n`), using an
/// optimal strategy (`strategy` has `n - 1` entries, e.g. from
/// [`crate::strategy::optimal`]) to amortise doublings against isogeny
/// evaluations.
///
/// `e1e2` need not be a literal product of the *same* curve twice: any
/// valid elliptic product with a maximal isotropic `2^n`-kernel works,
/// which is what lets `decaps_chain_theta` feed this the post-`prodker`
/// state of the decapsulation chain directly (`n = e - 4` when the
/// remaining generators have order `2^{e-2}`).
pub fn product_isogeny<F: Field + Send + Sync>(
    e1e2: &EllipticProduct<F>,
    p1p2: &CouplePoint<F>,
    q1q2: &CouplePoint<F>,
    image_points: &[CouplePoint<F>],
    n: usize,
    strategy: &[usize],
) -> (EllipticProduct<F>, Vec<CouplePoint<F>>) {
    let num_image_points = image_points.len();
    let mut kernel_couple_pts: Vec<CouplePoint<F>> = image_points.to_vec();
    kernel_couple_pts.push(*p1p2);
    kernel_couple_pts.push(*q1q2);

    let mut strat_idx = 0usize;
    let mut level: Vec<usize> = vec![0];
    let mut prev: usize = level.iter().sum();

    let mut ker1 = *p1p2;
    let mut ker2 = *q1q2;

    while prev != n - 1 {
        level.push(strategy[strat_idx]);
        let stepped = ep_double_pair(e1e2, &ker1, &ker2, strategy[strat_idx]);
        ker1 = stepped.0;
        ker2 = stepped.1;
        kernel_couple_pts.push(ker1);
        kernel_couple_pts.push(ker2);
        prev += strategy[strat_idx];
        strat_idx += 1;
    }
    kernel_couple_pts.pop();
    kernel_couple_pts.pop();
    level.pop();

    let (mut domain, mut kernel_pts) = gluing_isogeny(e1e2, &ker1, &ker2, &kernel_couple_pts);

    for k in 1..n {
        prev = level.iter().sum();
        let kernel_len = kernel_pts.len();
        let mut tp1 = kernel_pts[kernel_len - 2];
        let mut tp2 = kernel_pts[kernel_len - 1];

        while prev != (n - 1 - k) {
            level.push(strategy[strat_idx]);
            let steps = strategy[strat_idx];
            let stepped = domain.double_iter_pair(&tp1, &tp2, steps);
            tp1 = stepped.0;
            tp2 = stepped.1;
            kernel_pts.push(tp1);
            kernel_pts.push(tp2);
            prev += steps;
            strat_idx += 1;
        }
        kernel_pts.pop();
        kernel_pts.pop();
        level.pop();

        if k == n - 2 {
            domain = two_isogeny(&tp1, &tp2, &mut kernel_pts, [false, false]);
        } else if k == n - 1 {
            domain = two_isogeny_to_product(&tp1, &tp2, &mut kernel_pts);
        } else {
            domain = two_isogeny(&tp1, &tp2, &mut kernel_pts, [false, true]);
        }
    }

    domain = splitting_isomorphism(domain, &mut kernel_pts);
    split_to_product(&domain, &kernel_pts, num_image_points)
}
