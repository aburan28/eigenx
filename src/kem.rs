//! The Eigen-Séta KEM.
//!
//! * `keygen` is variable time (key generation is assumed to run in a
//!   trusted environment; see SIDE_CHANNELS.md).
//! * `encaps` and `decaps` are constant time in all secret data (the sender's
//!   kernel scalar and masks; the receiver's Kani scalars): fixed public
//!   loop counts, branch-free formulas, constant-time selections.  Branches
//!   in these functions depend only on public data (parameters, the public
//!   key, the ciphertext's validity as curve points).
//! * Fujisaki–Okamoto-style re-encryption with implicit rejection on the
//!   deterministic trapdoor function s -> (E_C, [a]phi(P), [b]phi(Q)),
//!   (a, b) = G(s, pk). IND-CCA security has not been established.

use crate::chain::{decaps_chain_theta, first_step_glues, keygen_chain, keygen_chain_theta, Pair};
use crate::cm::{add_vartime, eigenvector_2k, mul_vartime, point_from_x, torsion_basis_cof, Omega};
use crate::ct::{Choice, ConditionallySelectable, ConstantTimeEq};
use crate::field::Field;
use crate::fp::Fp;
use crate::fp2::Fp2;
use crate::isog3::chain3;
use crate::mont::{
    ladder, ladder3pt, recover_y, xadd, xdbl, xdbl_iter, xtpl, xtpl_iter, MontCurve, ProjCurve,
    XPoint,
};
use crate::ntheory::{
    cornacchia, is_probable_prime, modpos, random_below, sqrt_mod_2power, two_squares, v2,
};
use crate::pairing::{dlog_3k, full_order_3k, miller_many, weil_ratio, weil_sq};
use crate::params::ParamSet;
use crate::scalar::{OddModulus, Scalar};
use crate::weier::{WCurve, WIso, WPoint};
use core::marker::PhantomData;
use num_bigint::{BigInt, BigUint};
use num_traits::{One, Zero};
use rand_core::{CryptoRng, RngCore};
use sha3::digest::{ExtendableOutput, Update, XofReader};
use sha3::Shake256;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const SS_BYTES: usize = 32;

#[derive(Clone, Debug)]
pub struct PublicKey {
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Ciphertext {
    pub bytes: Vec<u8>,
}

/// Secret key.  `lam_s`, `lam_t`, `cp2` are mod 2^E; `c3`, `cp3` mod 3^F3.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretKey {
    lam_s: Scalar,
    lam_t: Scalar,
    cp2: Scalar,
    c3: Scalar,
    cp3: Scalar,
    z: [u8; 32],
    #[zeroize(skip)]
    pk: Vec<u8>,
    #[zeroize(skip)]
    pkh: [u8; 32],
    /// Bit length of the secret degree f (public by design: f is drawn from
    /// [f_lo, 2 f_lo)).  f itself is not retained: decapsulation never uses it.
    #[zeroize(skip)]
    f_bits: u64,
}

impl SecretKey {
    pub fn secret_degree_bits(&self) -> u64 {
        self.f_bits
    }
    /// The public key this secret key belongs to.
    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            bytes: self.pk.clone(),
        }
    }
}

/// Redacting Debug: never print secret scalars.
impl core::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SecretKey")
            .field("pkh", &self.pkh)
            .finish_non_exhaustive()
    }
}

fn shake(parts: &[&[u8]], out: usize) -> Vec<u8> {
    let mut h = Shake256::default();
    for p in parts {
        h.update(p);
    }
    let mut r = h.finalize_xof();
    let mut v = vec![0u8; out];
    r.read(&mut v);
    v
}

pub struct EigenSeta<P: ParamSet<N>, const N: usize>(PhantomData<P>);

/// Canonical basis of E_A[3^F3] (public, derived from the public key).
#[derive(Clone, Copy, Debug)]
pub struct PkBasis<Fl: Field> {
    pd: WPoint<Fl>,
    qd: WPoint<Fl>,
    pqd: WPoint<Fl>,
}

/// Public data derived from (pk, ct) by `decaps_prepare`.
#[derive(Clone, Copy, Debug)]
pub struct DecapsPrep<Fl: Field> {
    pkf: [Fl; 4],
    a_c: Fl,
    s_pt: WPoint<Fl>,
    t_pt: WPoint<Fl>,
    x1: WPoint<Fl>,
    x2: WPoint<Fl>,
    pk_basis: PkBasis<Fl>,
}

type F<P, const N: usize> = Fp2<<P as ParamSet<N>>::Cfg, N>;

/// Montgomery coefficient of E_A and lifts of the public 2-power basis
/// (`EigenSeta::public_basis_2`).
pub type PublicBasis2<P, const N: usize> = (F<P, N>, WPoint<F<P, N>>, WPoint<F<P, N>>);
type WPair<P, const N: usize> = (WPoint<F<P, N>>, WPoint<F<P, N>>);

/// Public parameters derived at run time.
struct Ctx<P: ParamSet<N>, const N: usize> {
    d: BigUint,
    d_bits: usize,
    e: usize,
    e_limbs: usize,
    modd: OddModulus,
    _p: PhantomData<P>,
}

impl<P: ParamSet<N>, const N: usize> Ctx<P, N> {
    fn new() -> Self {
        let d = BigUint::from(3u32).pow(P::F3 as u32);
        let d_bits = d.bits() as usize;
        let modd = OddModulus::new(&d);
        Ctx {
            d,
            d_bits,
            e: P::E,
            e_limbs: P::E.div_ceil(64) + 1,
            modd,
            _p: PhantomData,
        }
    }
}

fn fp_from_hex<P: ParamSet<N>, const N: usize>(h: &str) -> Fp<P::Cfg, N> {
    let b = BigUint::parse_bytes(h.as_bytes(), 16).unwrap();
    let mut l = [0u64; N];
    for (i, x) in b.to_u64_digits().iter().enumerate() {
        l[i] = *x;
    }
    Fp::from_canonical_limbs(l)
}

/// Deterministic basis (P, Q) of E[3^F3] on the Montgomery curve y^2 = x^3 +
/// A x^2 + x, computed from public data only (variable time).
fn canonical_basis_3<P: ParamSet<N>, const N: usize>(a: &F<P, N>) -> Option<WPair<P, N>> {
    // A = ±2 makes the Montgomery model singular. Reject it before any
    // point arithmetic, including the public, variable-time basis search.
    if bool::from((a.square() - F::<P, N>::from_i64(4)).is_zero()) {
        return None;
    }
    let w = WCurve::from_montgomery(*a);
    let pc = ProjCurve::from_affine(*a);
    let cof_odd = BigUint::from(P::COFACTOR);
    let mut found: Vec<WPoint<F<P, N>>> = vec![];
    // An arbitrary encoded curve need not have the expected 3^F3 torsion.
    // Never let a caller-supplied public key or ciphertext loop forever.
    // (p+1) / 3^F3 = 2^(E+1) * c, so the cofactor is a straight doubling
    // chain plus a tiny odd multiplier. Order checks are x-only triplings.
    for k in 1..=256u64 {
        let x = <F<P, N> as Field>::from_counter(k);
        let Some(r0) = point_from_x(&w, x) else {
            continue;
        };
        let r = w.dbl_iter_mont(&r0, P::E + 1);
        let r = mul_vartime(&w, &r, &cof_odd);
        let r3 = xtpl_iter(&XPoint::from_affine(r.x), P::F3 - 1, &pc);
        if bool::from(r3.is_infinity()) {
            continue;
        }
        if !bool::from(xtpl(&r3, &pc).is_infinity()) {
            continue; // not in E[3^F3] on a malformed public curve
        }
        if let Some(p0) = found.first() {
            let p3 = xtpl_iter(&XPoint::from_affine(p0.x), P::F3 - 1, &pc);
            if bool::from(p3.x_eq(&r3)) {
                continue; // [3^(k-1)] R in <[3^(k-1)] P>
            }
            return Some((*p0, r));
        }
        found.push(r);
    }
    None
}

/// One y-sign pattern of the dual 3-isogeny. Same operation count for every
/// pattern. Returns the recovered scalar and whether the codomain is E_A.
fn sign_pattern<P: ParamSet<N>, const N: usize>(
    flip1: bool,
    flip2: bool,
    alpha1: WPoint<F<P, N>>,
    alpha2: WPoint<F<P, N>>,
    wc: WCurve<F<P, N>>,
    cx1: WPoint<F<P, N>>,
    cx2: WPoint<F<P, N>>,
    x1: WPoint<F<P, N>>,
    x2: WPoint<F<P, N>>,
    a_c: F<P, N>,
    a_a: F<P, N>,
    pd: WPoint<F<P, N>>,
    qd: WPoint<F<P, N>>,
    pcc: ProjCurve<F<P, N>>,
    pa: ProjCurve<F<P, N>>,
    wa: WCurve<F<P, N>>,
    d: &BigUint,
    k3: usize,
    d_bits: usize,
    limbs: usize,
) -> (Scalar, Choice) {
    let ctx = Ctx::<P, N>::new();
    let a1 = if flip1 { alpha1.neg() } else { alpha1 };
    let a2 = if flip2 { alpha2.neg() } else { alpha2 };
    let psi1 = wc.sub(&a1, &cx1);
    let psi2 = wc.sub(&a2, &cx2);
    let g1_full = !xtpl_iter(&XPoint::from_affine(psi1.x), k3 - 1, &pcc).is_infinity() & !psi1.inf;
    let g = WPoint::conditional_select(&psi2, &psi1, g1_full);
    let h = WPoint::conditional_select(&psi1, &psi2, g1_full);
    let xg = WPoint::conditional_select(&x2, &x1, g1_full);
    let xh = WPoint::conditional_select(&x1, &x2, g1_full);
    // Four Miller chains, one per base point, each evaluated at the other
    // two.  Together they are e(g, x1), e(g, x2), e(h, x1), e(h, x2).
    let (e_g1, e_g2, e_h1, e_h2) = std::thread::scope(|scope| {
        let mg = scope.spawn(|| miller_many(&wc, &g, &[x1, x2], d));
        let mh = scope.spawn(|| miller_many(&wc, &h, &[x1, x2], d));
        let mx1 = scope.spawn(|| miller_many(&wc, &x1, &[g, h], d));
        let mx2 = scope.spawn(|| miller_many(&wc, &x2, &[g, h], d));
        let mg = mg.join().expect("weil");
        let mh = mh.join().expect("weil");
        let mx1 = mx1.join().expect("weil");
        let mx2 = mx2.join().expect("weil");
        (
            weil_ratio(mg[0], mx1[0]),
            weil_ratio(mg[1], mx2[0]),
            weil_ratio(mh[0], mx1[1]),
            weil_ratio(mh[1], mx2[1]),
        )
    });
    let r1_full = full_order_3k(&e_g1, k3);
    let base = F::<P, N>::conditional_select(&e_g2, &e_g1, r1_full);
    let tgt = F::<P, N>::conditional_select(&e_h2, &e_h1, r1_full);
    let kk = dlog_3k(&base, &tgt, k3);
    let kk = ctx.modd.to_scalar_width(&ctx.modd.reduce(&kk), limbs);
    let khat = wc.sub(&xh, &smul_ct(&a_c, &xg, &kk, d_bits));
    let mut pts = [XPoint::from_affine(x1.x), XPoint::from_affine(x2.x)];
    let cod = chain3(&pcc, &XPoint::from_affine(khat.x), k3, &mut pts);
    let a_cod = cod.affine_a();
    let land = a_cod.ct_eq(&a_a) | a_cod.ct_eq(&(-a_a));
    let neg = a_cod.ct_eq(&(-a_a));
    let mut imx = [F::<P, N>::zero(); 2];
    for i in 0..2 {
        let x = pts[i].affine();
        imx[i] = F::<P, N>::conditional_select(&x, &(-x), neg);
    }
    let full0 = !xtpl_iter(&XPoint::from_affine(imx[0]), k3 - 1, &pa).is_infinity();
    let xkg = F::<P, N>::conditional_select(&imx[1], &imx[0], full0);
    let (yk, _) = wa.rhs(&xkg).sqrt();
    let kg = WPoint::new(xkg, yk);
    let (z1, z2) = std::thread::scope(|scope| {
        let m_kg = scope.spawn(|| miller_many(&wa, &kg, &[qd, pd], d));
        let m_qd = scope.spawn(|| miller_many(&wa, &qd, &[kg], d));
        let m_pd = scope.spawn(|| miller_many(&wa, &pd, &[kg], d));
        let m_kg = m_kg.join().expect("weil");
        let m_qd = m_qd.join().expect("weil");
        let m_pd = m_pd.join().expect("weil");
        (
            weil_ratio(m_kg[0], m_qd[0]),
            weil_ratio(m_pd[0], m_kg[1]),
        )
    });
    let s = dlog_3k(&z1, &z2, k3);
    (ctx.modd.to_scalar_width(&ctx.modd.reduce(&s), limbs), land)
}

/// Full point [k]P on the Montgomery curve (A), constant time in k and P:
/// ladder + Okeya–Sakurai y-recovery + one inversion.
fn smul_ct<Fl: Field>(a: &Fl, p: &WPoint<Fl>, k: &Scalar, nbits: usize) -> WPoint<Fl> {
    let pc = ProjCurve::from_affine(*a);
    let (r0, r1) = ladder(&XPoint::from_affine(p.x), k, nbits, &pc);
    let (x, y, z) = recover_y(a, &p.x, &p.y, &r0, &r1);
    let zi = z.invert();
    let r = WPoint {
        x: x * zi,
        y: y * zi,
        inf: Choice::from(0),
    };
    let r = WPoint::conditional_select(&r, &WPoint::infinity(), z.is_zero());
    // Okeya–Sakurai y-recovery has Z = 0 when [k+1]P = O. In that
    // case [k]P = -P, independently of the order of P. Select it without
    // branching on the secret scalar.
    WPoint::conditional_select(&r, &p.neg(), r1.is_infinity())
}

fn fbytes<Fl: Field>(xs: &[Fl]) -> Vec<u8> {
    let mut v = vec![];
    for x in xs {
        v.extend(x.to_bytes());
    }
    v
}

fn parse4<Fl: Field>(b: &[u8]) -> Option<[Fl; 4]> {
    let l = Fl::byte_len();
    if b.len() != 4 * l {
        return None;
    }
    let mut out = [Fl::zero(); 4];
    for i in 0..4 {
        let (v, ok) = Fl::from_bytes(&b[i * l..(i + 1) * l]);
        if !bool::from(ok) {
            return None;
        }
        out[i] = v;
    }
    Some(out)
}

impl<P: ParamSet<N>, const N: usize> EigenSeta<P, N> {
    pub fn name() -> &'static str {
        P::NAME
    }
    pub fn pk_bytes() -> usize {
        4 * <F<P, N> as Field>::byte_len()
    }
    pub fn ct_bytes() -> usize {
        4 * <F<P, N> as Field>::byte_len()
    }

    /// Serialised secret-key length: three scalars mod 2^E, two mod 3^F3,
    /// the 32-byte rejection seed, the 8-byte degree bit length and the
    /// public key.
    pub fn sk_bytes() -> usize {
        let ctx = Ctx::<P, N>::new();
        3 * ctx.e.div_ceil(8) + 2 * ctx.d_bits.div_ceil(8) + 32 + 8 + Self::pk_bytes()
    }

    /// Serialise a secret key (little-endian scalars, then `z`, `f_bits`,
    /// public key).  The caller is responsible for zeroising the output.
    pub fn sk_to_bytes(sk: &SecretKey) -> Vec<u8> {
        let ctx = Ctx::<P, N>::new();
        let eb = ctx.e.div_ceil(8);
        let db = ctx.d_bits.div_ceil(8);
        let mut v = Vec::with_capacity(Self::sk_bytes());
        v.extend(sk.lam_s.to_le_bytes(eb));
        v.extend(sk.lam_t.to_le_bytes(eb));
        v.extend(sk.cp2.to_le_bytes(eb));
        v.extend(sk.c3.to_le_bytes(db));
        v.extend(sk.cp3.to_le_bytes(db));
        v.extend_from_slice(&sk.z);
        v.extend_from_slice(&sk.f_bits.to_le_bytes());
        v.extend_from_slice(&sk.pk);
        v
    }

    /// Parse a secret key written by `sk_to_bytes`.  Returns `None` on a
    /// length mismatch or an unparsable embedded public key.  The scalars
    /// are reduced to their moduli so that a corrupted file cannot produce
    /// out-of-range secret values.
    pub fn sk_from_bytes(b: &[u8]) -> Option<SecretKey> {
        if b.len() != Self::sk_bytes() {
            return None;
        }
        let ctx = Ctx::<P, N>::new();
        let eb = ctx.e.div_ceil(8);
        let db = ctx.d_bits.div_ceil(8);
        let el = ctx.e_limbs;
        let dl = ctx.modd.limbs();
        let mut off = 0usize;
        let mut take = |n: usize| {
            let s = &b[off..off + n];
            off += n;
            s
        };
        let s2 = |s: &[u8]| Scalar::from_le_bytes(s, el).mod_2k(ctx.e);
        let lam_s = s2(take(eb));
        let lam_t = s2(take(eb));
        let cp2 = s2(take(eb));
        let s3 = |s: &[u8]| {
            let raw = Scalar::from_le_bytes(s, dl + 1);
            ctx.modd.to_scalar_width(&ctx.modd.reduce(&raw), dl)
        };
        let c3 = s3(take(db));
        let cp3 = s3(take(db));
        let mut z = [0u8; 32];
        z.copy_from_slice(take(32));
        let mut fb = [0u8; 8];
        fb.copy_from_slice(take(8));
        let f_bits = u64::from_le_bytes(fb);
        let pk = take(Self::pk_bytes()).to_vec();
        let pkf = parse4::<F<P, N>>(&pk)?;
        if !Self::pk_points_valid(&pkf) {
            return None;
        }
        let pkh = shake(&[b"EigenSeta-P", &pk], 32);
        let mut pkh_a = [0u8; 32];
        pkh_a.copy_from_slice(&pkh);
        Some(SecretKey {
            lam_s,
            lam_t,
            cp2,
            c3,
            cp3,
            z,
            pk,
            pkh: pkh_a,
            f_bits,
        })
    }

    /// The range [lo, 2 lo) from which the secret degree f is drawn.
    pub fn secret_degree_range() -> (BigUint, BigUint) {
        let lo = BigUint::parse_bytes(P::F_LO_HEX.as_bytes(), 16).unwrap();
        let hi = &lo * 2u32;
        (lo, hi)
    }

    /// The square root of -7 modulo 2^E used to split the eigenvalues of ω
    /// (public).
    fn sqrt_minus7_mod_2e(ctx: &Ctx<P, N>) -> BigUint {
        let e = ctx.e;
        let n2 = BigUint::one() << e;
        let s_d = sqrt_mod_2power(&((BigUint::one() << (e + 2)) - 7u32), e as u64 + 2);
        s_d % &n2
    }

    /// Kani scalars (c, c') for a candidate secret degree f, with the sign
    /// choice that makes the fixed decapsulation schedule valid, or `None`
    /// if f is not admissible (even, divisible by 3 or 7, 2f ≡ 2^E mod d,
    /// 2^E − 7 d² f² not a usable sum of two squares, or no sign choice fits
    /// the schedule).  Variable time; key generation and cryptanalysis only.
    pub fn kani_scalars_for_degree<R: RngCore + CryptoRng>(
        f: &BigUint,
        flip: bool,
        rng: &mut R,
    ) -> Option<(BigInt, BigInt)> {
        let ctx = Ctx::<P, N>::new();
        let n2 = BigUint::one() << ctx.e;
        let d = &ctx.d;
        if (f % 2u32).is_zero() || (f % 3u32).is_zero() || (f % 7u32).is_zero() {
            return None;
        }
        let two_f = BigInt::from(f.clone()) * 2 - BigInt::from(n2.clone());
        if modpos(&two_f, d).is_zero() {
            return None;
        }
        let m7 = d * d * f * f * 7u32;
        if m7 >= n2 {
            return None;
        }
        let m = &n2 - m7;
        let (x, y) = two_squares(&m, rng, 1 << 14)?;
        let (c0, cp0) = if (&x % 2u32).is_zero() {
            (x, y)
        } else {
            (y, x)
        };
        let s_dm = Self::sqrt_minus7_mod_2e(&ctx);
        let s_dm = if flip { &n2 - &s_dm } else { s_dm };
        let s0 = (f * &s_dm) % &n2;
        let ds0 = BigInt::from((d * &s0) % &n2);
        let mut choice = None;
        for sc in [1i32, -1] {
            for scp in [1i32, -1] {
                let c = BigInt::from(c0.clone()) * sc;
                let cp = BigInt::from(cp0.clone()) * scp;
                let lam_s = &ds0 + &c;
                if v2(&(&cp - &lam_s)).max(v2(&(&cp + &lam_s))) == 2 {
                    choice = Some((c.clone(), cp.clone()));
                }
            }
        }
        choice
    }

    /// Assemble a secret key from the secret degree f, its Kani scalars and
    /// the public key.  `flip` swaps the roles of the two public basis points
    /// (the sign of √−7); honest key generation uses `flip = false`.
    fn secret_key_from_parts(
        pk_bytes: &[u8],
        f: &BigUint,
        c: &BigInt,
        cp: &BigInt,
        flip: bool,
        z: [u8; 32],
    ) -> SecretKey {
        let ctx = Ctx::<P, N>::new();
        let n2 = BigUint::one() << ctx.e;
        let d = &ctx.d;
        let s_dm = Self::sqrt_minus7_mod_2e(&ctx);
        let s_dm = if flip { &n2 - &s_dm } else { s_dm };
        let s0 = (f * &s_dm) % &n2;
        let ds0 = BigInt::from((d * &s0) % &n2);
        let lam_s = modpos(&(&ds0 + c), &n2);
        let lam_t = modpos(&(c - &ds0), &n2);
        let pkh = shake(&[b"EigenSeta-P", pk_bytes], 32);
        let mut pkh_a = [0u8; 32];
        pkh_a.copy_from_slice(&pkh);
        let el = ctx.e_limbs;
        let dl = ctx.modd.limbs();
        SecretKey {
            lam_s: Scalar::from_biguint(&lam_s, el),
            lam_t: Scalar::from_biguint(&lam_t, el),
            cp2: Scalar::from_biguint(&modpos(cp, &n2), el),
            c3: Scalar::from_biguint(&modpos(c, d), dl),
            cp3: Scalar::from_biguint(&modpos(cp, d), dl),
            z,
            pk: pk_bytes.to_vec(),
            pkh: pkh_a,
            f_bits: f.bits(),
        }
    }

    /// The secret key is a function of the secret degree f and public data.
    /// Given a public key and a candidate f, return the (at most two) secret
    /// keys consistent with it: one per assignment of the two public basis
    /// points to the eigenvalues.  An attacker who learns f recovers the key
    /// by testing both candidates against one self-made ciphertext.  Used by
    /// the cryptanalysis module; variable time.
    pub fn secret_keys_from_degree<R: RngCore + CryptoRng>(
        pk: &PublicKey,
        f: &BigUint,
        rng: &mut R,
    ) -> Vec<SecretKey> {
        let mut z = [0u8; 32];
        rng.fill_bytes(&mut z);
        [false, true]
            .into_iter()
            .filter_map(|flip| {
                let (c, cp) = Self::kani_scalars_for_degree(f, flip, rng)?;
                Some(Self::secret_key_from_parts(&pk.bytes, f, &c, &cp, flip, z))
            })
            .collect()
    }

    fn ed_curve() -> WCurve<F<P, N>> {
        WCurve::new(
            F::<P, N>::ZERO,
            Fp2::from_fp(fp_from_hex::<P, N>(P::ED_A4_HEX)),
            Fp2::from_fp(fp_from_hex::<P, N>(P::ED_A6_HEX)),
        )
    }

    /// Masks (a, b), odd, mod 2^E, from the kernel scalar s.
    fn masks(ctx: &Ctx<P, N>, s_bytes: &[u8], pkh: &[u8]) -> (Scalar, Scalar) {
        let nb = ctx.e.div_ceil(8);
        let h = shake(&[b"EigenSeta-G", s_bytes, pkh], 2 * nb);
        let mut a = Scalar::from_le_bytes(&h[..nb], ctx.e_limbs).mod_2k(ctx.e);
        let mut b = Scalar::from_le_bytes(&h[nb..], ctx.e_limbs).mod_2k(ctx.e);
        a.l[0] |= 1;
        b.l[0] |= 1;
        (a, b)
    }

    fn s_to_bytes(ctx: &Ctx<P, N>, s: &Scalar) -> Vec<u8> {
        s.to_le_bytes(ctx.d_bits.div_ceil(8))
    }

    /// Deterministic encryption of the kernel scalar s.  Constant time in s.
    /// Public, variable-time: the canonical basis of E_A[3^F3] and x(P - Q).
    fn pk_basis(a_a: &F<P, N>) -> Option<PkBasis<F<P, N>>> {
        let (pd, qd) = canonical_basis_3::<P, N>(a_a)?;
        let pqd = WCurve::from_montgomery(*a_a).sub(&pd, &qd);
        Some(PkBasis { pd, qd, pqd })
    }

    /// The Montgomery coefficient of E_A and lifts (P_A, Q_A) of the public
    /// 2-power basis with x(P_A − Q_A) as published.  Public inputs only;
    /// used by the cryptanalysis module.  `None` on a malformed public key.
    pub fn public_basis_2(pk: &PublicKey) -> Option<PublicBasis2<P, N>> {
        let [a, xp, xq, xpq] = parse4::<F<P, N>>(&pk.bytes)?;
        let curve = WCurve::from_montgomery(a);
        let p = point_from_x(&curve, xp)?;
        let mut q = point_from_x(&curve, xq)?;
        if !bool::from(curve.sub(&p, &q).x.ct_eq(&xpq)) {
            q = q.neg();
            if !bool::from(curve.sub(&p, &q).x.ct_eq(&xpq)) {
                return None;
            }
        }
        Some((a, p, q))
    }

    /// Check the public 2^E-basis and its differential point before an
    /// untrusted public key can enter deterministic encapsulation.
    fn pk_points_valid(pk: &[F<P, N>; 4]) -> bool {
        let [a, xp, xq, xpq] = *pk;
        if bool::from((a.square() - F::<P, N>::from_i64(4)).is_zero()) {
            return false;
        }
        let curve = WCurve::from_montgomery(a);
        let Some(p) = point_from_x(&curve, xp) else {
            return false;
        };
        let Some(q) = point_from_x(&curve, xq) else {
            return false;
        };
        let diff = curve.sub(&p, &q);
        let sum = curve.add(&p, &q);
        if !bool::from(diff.x.ct_eq(&xpq)) && !bool::from(sum.x.ct_eq(&xpq)) {
            return false;
        }
        let pc = ProjCurve::from_affine(a);
        let (hp, hq) = std::thread::scope(|scope| {
            let p = scope.spawn(|| xdbl_iter(&XPoint::from_affine(xp), P::E - 1, &pc));
            let q = scope.spawn(|| xdbl_iter(&XPoint::from_affine(xq), P::E - 1, &pc));
            (p.join().expect("order"), q.join().expect("order"))
        });
        !bool::from(hp.is_infinity())
            && !bool::from(hq.is_infinity())
            && bool::from(xdbl(&hp, &pc).is_infinity())
            && bool::from(xdbl(&hq, &pc).is_infinity())
            && !bool::from(hp.affine().ct_eq(&hq.affine()))
    }

    fn enc_det(ctx: &Ctx<P, N>, pk: &[F<P, N>; 4], s: &Scalar, pkh: &[u8]) -> Option<Vec<u8>> {
        Some(Self::enc_det_core(
            ctx,
            pk,
            &Self::pk_basis(&pk[0])?,
            s,
            pkh,
        ))
    }

    /// Deterministic encryption given the public basis: constant time in `s`.
    fn enc_det_core(
        ctx: &Ctx<P, N>,
        pk: &[F<P, N>; 4],
        basis: &PkBasis<F<P, N>>,
        s: &Scalar,
        pkh: &[u8],
    ) -> Vec<u8> {
        let a_a = pk[0];
        let pc = ProjCurve::from_affine(a_a);
        let PkBasis { pd, qd, pqd } = *basis;
        let xk = ladder3pt(
            &XPoint::from_affine(pd.x),
            &XPoint::from_affine(qd.x),
            &XPoint::from_affine(pqd.x),
            s,
            ctx.d_bits,
            &pc,
        );
        let mut pts = [
            XPoint::from_affine(pk[1]),
            XPoint::from_affine(pk[2]),
            XPoint::from_affine(pk[3]),
        ];
        let cod = chain3(&pc, &xk, P::F3, &mut pts);
        let a_c = cod.affine_a();
        let cc = ProjCurve::from_affine(a_c);
        let (xp, xq, xpq) = (pts[0], pts[1], pts[2]);
        let xppq = xadd(&xp, &xq, &xpq); // x(phiP + phiQ)
        let (a, b) = Self::masks(ctx, &Self::s_to_bytes(ctx, s), pkh);
        let m = b.mul_mod_2k(&a.inv_mod_2k(ctx.e), ctx.e);
        // Three independent 2^E ladders: [a]phiP, [b]phiQ, and [a](phiP-[b/a]phiQ).
        let ((xs, _), (xt, _), (xst, _)) = std::thread::scope(|scope| {
            let s = scope.spawn(|| ladder(&xp, &a, ctx.e, &cc));
            let t = scope.spawn(|| ladder(&xq, &b, ctx.e, &cc));
            let u = scope.spawn(|| {
                let wpt = ladder3pt(&xp, &xq, &xppq, &m, ctx.e, &cc);
                ladder(&wpt, &a, ctx.e, &cc)
            });
            (
                s.join().expect("ladder"),
                t.join().expect("ladder"),
                u.join().expect("ladder"),
            )
        });
        fbytes(&[a_c, xs.affine(), xt.affine(), xst.affine()])
    }

    pub fn encaps<R: RngCore + CryptoRng>(
        pk: &PublicKey,
        rng: &mut R,
    ) -> Option<(Ciphertext, [u8; SS_BYTES])> {
        let ctx = Ctx::<P, N>::new();
        let pkf = parse4::<F<P, N>>(&pk.bytes)?;
        if !Self::pk_points_valid(&pkf) {
            return None;
        }
        let pkh = shake(&[b"EigenSeta-P", &pk.bytes], 32);
        // Uniform seed reduced mod d = 3^F3. A full-width draw is
        // ceil(d_bits/8) bytes; ceil(λ/8) extra bytes makes the statistical
        // distance of that reduction at most 2^{-λ}
        // (SD ≤ m/2^{b+2} < 2^{-λ-2}). L1 (λ = 128) stays at +16 bytes.
        let seed_bytes = ctx.d_bits.div_ceil(8) + P::LAMBDA.div_ceil(8);
        let mut buf = vec![0u8; seed_bytes];
        rng.fill_bytes(&mut buf);
        let s_raw = Scalar::from_le_bytes(&buf, seed_bytes.div_ceil(8));
        let s = ctx
            .modd
            .to_scalar_width(&ctx.modd.reduce(&s_raw), ctx.modd.limbs());
        buf.zeroize();
        let ct = Self::enc_det(&ctx, &pkf, &s, &pkh)?;
        let k = shake(
            &[b"EigenSeta-K", &Self::s_to_bytes(&ctx, &s), &ct],
            SS_BYTES,
        );
        let mut out = [0u8; SS_BYTES];
        out.copy_from_slice(&k);
        Some((Ciphertext { bytes: ct }, out))
    }

    pub fn decaps(sk: &SecretKey, ct: &Ciphertext) -> [u8; SS_BYTES] {
        match Self::decaps_prepare(sk, ct) {
            Some(prep) => Self::decaps_core(sk, &prep, ct),
            // public malformation: the implicit-rejection key
            None => {
                let mut out = [0u8; SS_BYTES];
                out.copy_from_slice(&shake(&[b"EigenSeta-R", &sk.z, &ct.bytes], SS_BYTES));
                out
            }
        }
    }

    /// Public phase of decapsulation (variable time, depends only on the public
    /// key and the ciphertext): parse, lift S and T with their relative sign,
    /// and compute the canonical 3-torsion bases of E_C and E_A.
    pub fn decaps_prepare(sk: &SecretKey, ct: &Ciphertext) -> Option<DecapsPrep<F<P, N>>> {
        let [a_c, xs, xt, xst] = parse4::<F<P, N>>(&ct.bytes)?;
        if bool::from((a_c.square() - F::<P, N>::from_i64(4)).is_zero()) {
            return None;
        }
        let pkf = parse4::<F<P, N>>(&sk.pk)?;
        let wc = WCurve::from_montgomery(a_c);
        let s_pt = point_from_x(&wc, xs)?;
        let mut t_pt = point_from_x(&wc, xt)?;
        if !bool::from(wc.sub(&s_pt, &t_pt).x.ct_eq(&xst)) {
            t_pt = t_pt.neg();
            if !bool::from(wc.sub(&s_pt, &t_pt).x.ct_eq(&xst)) {
                return None;
            }
        }
        // The two 3^F3 bases are independent public work.
        let (basis_c, pk_basis) = std::thread::scope(|scope| {
            let b = scope.spawn(|| canonical_basis_3::<P, N>(&a_c));
            let p = scope.spawn(|| Self::pk_basis(&pkf[0]));
            (b.join().expect("basis"), p.join().expect("pk basis"))
        });
        let (x1, x2) = basis_c?;
        Some(DecapsPrep {
            pkf,
            a_c,
            s_pt,
            t_pt,
            x1,
            x2,
            pk_basis: pk_basis?,
        })
    }

    /// Secret phase of decapsulation: trapdoor inversion, deterministic
    /// re-encryption, comparison and selection.  Intended to be constant
    /// time in the secret key and in s; kept out of line so that
    /// instruction-count tools can measure it on its own (see
    /// examples/ct_probe.rs and SIDE_CHANNELS.md).
    #[inline(never)]
    pub fn decaps_core(
        sk: &SecretKey,
        prep: &DecapsPrep<F<P, N>>,
        ct: &Ciphertext,
    ) -> [u8; SS_BYTES] {
        let reject = shake(&[b"EigenSeta-R", &sk.z, &ct.bytes], SS_BYTES);
        let mut out = [0u8; SS_BYTES];
        out.copy_from_slice(&reject);
        let ctx = Ctx::<P, N>::new();
        let s = Self::recover_s_core(sk, prep);
        let ct2 = Self::enc_det_core(&ctx, &prep.pkf, &prep.pk_basis, &s, &sk.pkh);
        let same = crate::ct::bytes_eq(&ct2, &ct.bytes);
        let k = shake(
            &[b"EigenSeta-K", &Self::s_to_bytes(&ctx, &s), &ct.bytes],
            SS_BYTES,
        );
        for i in 0..SS_BYTES {
            out[i] = u8::conditional_select(&out[i], &k[i], same);
        }
        out
    }

    /// The trapdoor inversion: recover s from a ciphertext.  Constant time in
    /// the secret key; returns None only on public malformation of `ct`.
    pub fn recover_s(sk: &SecretKey, ct: &Ciphertext) -> Option<Scalar> {
        Some(Self::recover_s_core(sk, &Self::decaps_prepare(sk, ct)?))
    }

    fn recover_s_core(sk: &SecretKey, prep: &DecapsPrep<F<P, N>>) -> Scalar {
        let ctx = Ctx::<P, N>::new();
        let DecapsPrep {
            pkf,
            a_c,
            s_pt,
            t_pt,
            x1,
            x2,
            ref pk_basis,
        } = *prep;
        let a_a = pkf[0];
        let wc = WCurve::from_montgomery(a_c);
        // secret: Kani kernel generators (c'S, -lamS S), (c'T, -lamT T)
        let e = ctx.e;
        let d_bits = ctx.d_bits;
        // Four secret ladders on E_C, independent of each other.
        let (k1a, k1b, k2a, k2b) = std::thread::scope(|scope| {
            let a = scope.spawn(|| smul_ct(&a_c, &s_pt, &sk.cp2, e));
            let b = scope.spawn(|| smul_ct(&a_c, &s_pt, &sk.lam_s, e));
            let c = scope.spawn(|| smul_ct(&a_c, &t_pt, &sk.cp2, e));
            let d = scope.spawn(|| smul_ct(&a_c, &t_pt, &sk.lam_t, e));
            (
                a.join().expect("ladder"),
                b.join().expect("ladder"),
                c.join().expect("ladder"),
                d.join().expect("ladder"),
            )
        });
        let k1 = (k1a, k1b.neg());
        let k2 = (k2a, k2b.neg());
        let o = WPoint::infinity();
        // Theta (2,2)-chain. Split recovers points from
        // the Kummer line (y-signs ambiguous) and may swap product factors per
        // payload point; identify α by matching −c′X's x-coordinate on each
        // image independently, then CT-select among α y-sign patterns.
        // The four 3-power ladders do not depend on the theta chain.
        let (fc, imgs, cpx1, cpx2, cx1, cx2) = std::thread::scope(|scope| {
            let cpx1 = scope.spawn(|| smul_ct(&a_c, &x1, &sk.cp3, d_bits));
            let cpx2 = scope.spawn(|| smul_ct(&a_c, &x2, &sk.cp3, d_bits));
            let cx1 = scope.spawn(|| smul_ct(&a_c, &x1, &sk.c3, d_bits));
            let cx2 = scope.spawn(|| smul_ct(&a_c, &x2, &sk.c3, d_bits));
            // (O, X1) is not read after the chain; only the images of (X1, O)
            // and (X2, O) identify α.
            let chain = decaps_chain_theta(&wc, &k1, &k2, e, &[(x1, o), (x2, o)]);
            (
                chain.0,
                chain.1,
                cpx1.join().expect("ladder"),
                cpx2.join().expect("ladder"),
                cx1.join().expect("ladder"),
                cx2.join().expect("ladder"),
            )
        });
        let sig = WIso::find(&fc, &wc);
        let mut alpha1 = WPoint::infinity();
        let mut alpha2 = WPoint::infinity();
        // Phi(X,0) = (αX, −c′X) up to component order and iso sign.
        let pick_alpha = |pr: &Pair<F<P, N>>, cpx_neg: &WPoint<F<P, N>>, sc: &WIso<F<P, N>>| {
            let u = sc.eval(&pr.0);
            let v = sc.eval(&pr.1);
            let match_u = u.x.ct_eq(&cpx_neg.x) & !u.inf;
            let match_v = v.x.ct_eq(&cpx_neg.x) & !v.inf;
            // match_u → α = v; match_v → α = u.
            let alpha = WPoint::conditional_select(&u, &v, match_u);
            let alpha = WPoint::conditional_select(&alpha, &u, match_v);
            (alpha, match_u | match_v)
        };
        for sc in [sig, sig.negate()] {
            let (a1, ok1) = pick_alpha(&imgs[0], &cpx1.neg(), &sc);
            let (a2, ok2) = pick_alpha(&imgs[1], &cpx2.neg(), &sc);
            let ok = ok1 & ok2;
            alpha1 = WPoint::conditional_select(&alpha1, &a1, ok);
            alpha2 = WPoint::conditional_select(&alpha2, &a2, ok);
        }
        let k3 = P::F3;
        let d = &ctx.d;
        let pcc = ProjCurve::from_affine(a_c);
        let pa = ProjCurve::from_affine(a_a);
        let wa = WCurve::from_montgomery(a_a);
        let (pd, qd) = (pk_basis.pd, pk_basis.qd);
        let limbs = ctx.modd.limbs();
        let d = d.clone();
        // Four y-sign patterns for (alpha1, alpha2). Each pattern is the same
        // amount of work; they share no writes, so they run together. A later
        // pattern still overrides an earlier one when both land, matching the
        // sequential select.
        let patterns = [(false, false), (false, true), (true, false), (true, true)];
        let found = std::thread::scope(|scope| {
            let mut joins = Vec::with_capacity(4);
            for (flip1, flip2) in patterns {
                let d = d.clone();
                joins.push(scope.spawn(move || {
                    sign_pattern::<P, N>(
                        flip1, flip2, alpha1, alpha2, wc, cx1, cx2, x1, x2, a_c, a_a, pd, qd, pcc,
                        pa, wa, &d, k3, d_bits, limbs,
                    )
                }));
            }
            joins
                .into_iter()
                .map(|j| j.join().expect("sign pattern"))
                .collect::<Vec<_>>()
        });
        let mut best = Scalar::zero(limbs);
        for (s, land) in found {
            for i in 0..limbs {
                best.l[i] = u64::conditional_select(&best.l[i], &s.l[i], land);
            }
        }
        best
    }

    /// Key generation (variable time).
    pub fn keygen<R: RngCore + CryptoRng>(rng: &mut R) -> (PublicKey, SecretKey) {
        let (pk, sk, _f) = Self::keygen_with_trace(rng);
        (pk, sk)
    }

    /// Key generation that also returns the secret degree f.  For the
    /// cryptanalysis experiments, which need to know the true f to build
    /// near-miss guesses; the KEM itself never stores f.
    pub fn keygen_with_trace<R: RngCore + CryptoRng>(
        rng: &mut R,
    ) -> (PublicKey, SecretKey, BigUint) {
        let prep = Self::keygen_prep(rng);
        loop {
            if let Some(kp) = Self::keygen_attempt(rng, &prep) {
                return kp;
            }
        }
    }

    /// Parallel key generation (variable time): run up to `jobs` independent
    /// `keygen_attempt` loops on separate threads (each with its own `OsRng`).
    /// The first success wins; others stop.  `jobs ≤ 1` is a single-threaded
    /// attempt loop.  Speeds up rejection sampling at Q3/Q5 approximately
    /// linearly in the number of cores until attempts contend on RAM.
    pub fn keygen_with_jobs(jobs: usize) -> (PublicKey, SecretKey) {
        let (pk, sk, _) = Self::keygen_with_jobs_trace(jobs);
        (pk, sk)
    }

    /// Like [`keygen_with_jobs`], also returning the secret degree f.
    pub fn keygen_with_jobs_trace(jobs: usize) -> (PublicKey, SecretKey, BigUint) {
        use rand_core::OsRng;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::mpsc;
        use std::sync::Arc;
        use std::thread;

        let jobs = jobs.max(1);
        if jobs == 1 {
            return Self::keygen_with_trace(&mut OsRng);
        }
        let done = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let mut handles = Vec::with_capacity(jobs);
        for _ in 0..jobs {
            let done = Arc::clone(&done);
            let tx = tx.clone();
            handles.push(thread::spawn(move || {
                let mut rng = OsRng;
                let prep = Self::keygen_prep(&mut rng);
                while !done.load(Ordering::Relaxed) {
                    if let Some(kp) = Self::keygen_attempt(&mut rng, &prep) {
                        if !done.swap(true, Ordering::SeqCst) {
                            let _ = tx.send(kp);
                        }
                        return;
                    }
                }
            }));
        }
        drop(tx);
        let kp = rx.recv().expect("at least one keygen worker must succeed");
        done.store(true, Ordering::SeqCst);
        for h in handles {
            let _ = h.join();
        }
        kp
    }

    /// Deterministic encryption of a chosen kernel scalar s (mod 3^F3) under
    /// a public key: the trapdoor function itself, without the KEM wrapper.
    /// Public inputs only; used by the cryptanalysis module to confirm a
    /// recovered s and by known-answer tests.
    pub fn encrypt_scalar(pk: &PublicKey, s: &BigUint) -> Option<Ciphertext> {
        let ctx = Ctx::<P, N>::new();
        let pkf = parse4::<F<P, N>>(&pk.bytes)?;
        if !Self::pk_points_valid(&pkf) {
            return None;
        }
        let pkh = shake(&[b"EigenSeta-P", &pk.bytes], 32);
        let s = Scalar::from_biguint(&(s % &ctx.d), ctx.modd.limbs());
        Some(Ciphertext {
            bytes: Self::enc_det(&ctx, &pkf, &s, &pkh)?,
        })
    }

    /// FO masks `(a, b)` for a kernel scalar under `pk` (same derivation as
    /// encapsulation). Cryptanalysis only: lets the dim-4 / mask-guess
    /// experiments feed the honest masks without reimplementing `G`.
    pub fn masks_for_s(pk: &PublicKey, s: &BigUint) -> Option<(Scalar, Scalar)> {
        let ctx = Ctx::<P, N>::new();
        let pkh = shake(&[b"EigenSeta-P", &pk.bytes], 32);
        let s = Scalar::from_biguint(&(s % &ctx.d), ctx.modd.limbs());
        Some(Self::masks(&ctx, &Self::s_to_bytes(&ctx, &s), &pkh))
    }

    /// The kernel scalar s as a `BigUint` from a recovered `Scalar`.
    pub fn scalar_to_biguint(s: &Scalar) -> BigUint {
        s.to_biguint()
    }

    /// The canonical (public) basis of E[3^F3] on the Montgomery curve with
    /// coefficient `a`, as used by encapsulation.  `None` if the curve does
    /// not carry the expected torsion.
    pub fn public_basis_3(a: &F<P, N>) -> Option<WPair<P, N>> {
        canonical_basis_3::<P, N>(a)
    }

    /// Walk the decapsulation chain for `(sk, ct)` in variable time with
    /// structural checks (see `chain::ChainReport`).  `None` on a publicly
    /// malformed ciphertext.  Analysis only.
    pub fn inspect_decaps_chain(
        sk: &SecretKey,
        ct: &Ciphertext,
    ) -> Option<crate::chain::ChainReport> {
        let prep = Self::decaps_prepare(sk, ct)?;
        let ctx = Ctx::<P, N>::new();
        let wc = WCurve::from_montgomery(prep.a_c);
        let e = ctx.e;
        let k1 = (
            smul_ct(&prep.a_c, &prep.s_pt, &sk.cp2, e),
            smul_ct(&prep.a_c, &prep.s_pt, &sk.lam_s, e).neg(),
        );
        let k2 = (
            smul_ct(&prep.a_c, &prep.t_pt, &sk.cp2, e),
            smul_ct(&prep.a_c, &prep.t_pt, &sk.lam_t, e).neg(),
        );
        Some(crate::chain::inspect_chain(&wc, &k1, &k2, e))
    }

    /// Pairing order 3^k is enough to tell the two factors apart: f ≢ ±g
    /// (mod 3) so the values cannot collide.  k = 8 makes the Miller loop
    /// 13 bits instead of ~1.6 F3 bits.
    const PAIR_K: usize = 8;

    fn keygen_prep<R: RngCore + CryptoRng>(rng: &mut R) -> KeygenPrep<P, N> {
        let e = P::E;
        let n2 = BigUint::one() << e;
        let ed = Self::ed_curve();
        let om = Omega::find(&ed, rng);
        let t = om.trace;
        let s_d = BigInt::from(sqrt_mod_2power(
            &((BigUint::one() << (e + 2)) - 7u32),
            e as u64 + 2,
        ));
        let mu = modpos(&((BigInt::from(t) + &s_d) / 2), &n2);
        let nu = modpos(&((BigInt::from(t) - &s_d) / 2), &n2);
        let p1 = crate::cm::p_plus_1::<F<P, N>>();
        let pchar = &p1 - 1u32;
        let v2p = p1.trailing_zeros().unwrap_or(0);
        let use_theta = v2p >= (e as u64) + 2;
        let level = if use_theta { e + 2 } else { e };
        let pair_k = Self::PAIR_K.min(P::F3);
        let (lo, _) = Self::secret_degree_range();
        KeygenPrep {
            ed,
            om,
            t,
            pchar,
            mu,
            nu,
            n2,
            e,
            use_theta,
            level,
            cof2: &p1 / (BigUint::one() << level),
            cof3: &p1 / BigUint::from(3u32).pow(P::F3 as u32),
            pair_cof: BigUint::from(3u32).pow((P::F3 - pair_k) as u32),
            pair_n: BigUint::from(3u32).pow(pair_k as u32),
            lo,
        }
    }

    fn signed_pt(p: WPoint<F<P, N>>, neg: bool) -> WPoint<F<P, N>> {
        if neg {
            p.neg()
        } else {
            p
        }
    }

    fn project_ev(
        ed: &WCurve<F<P, N>>,
        om: &Omega<F<P, N>>,
        r: &WPoint<F<P, N>>,
        other: &BigUint,
        half: &BigUint,
    ) -> Option<WPoint<F<P, N>>> {
        let p = add_vartime(ed, &om.eval(r), &mul_vartime(ed, r, other).neg());
        if bool::from(mul_vartime(ed, &p, half).inf) {
            None
        } else {
            Some(p)
        }
    }

    /// Recover (eps_P, eps_Q) and their order-2 images from Kummer-split
    /// points of (P+R, Q+R, R) by trying y-signs.  The honest images satisfy
    /// the test for exactly one pattern up to a global sign.
    fn recover_eigen(
        e: &WCurve<F<P, N>>,
        a0: WPoint<F<P, N>>,
        a1: WPoint<F<P, N>>,
        a2: WPoint<F<P, N>>,
        e_log: usize,
    ) -> Option<EigenRec<P, N>> {
        for s2 in [false, true] {
            let c = Self::signed_pt(a2, s2);
            for s0 in [false, true] {
                let a = Self::signed_pt(a0, s0);
                let p = e.sub(&a, &c);
                let hp = e.dbl_iter(&p, e_log - 1);
                if bool::from(hp.inf) || !bool::from(hp.y.is_zero()) {
                    continue;
                }
                for s1 in [false, true] {
                    let b = Self::signed_pt(a1, s1);
                    let q = e.sub(&b, &c);
                    let hq = e.dbl_iter(&q, e_log - 1);
                    if !bool::from(hq.inf)
                        && bool::from(hq.y.is_zero())
                        && !bool::from(hp.x.ct_eq(&hq.x))
                    {
                        return Some((p, q, hp, hq));
                    }
                }
            }
        }
        None
    }

    fn keygen_attempt<R: RngCore + CryptoRng>(
        rng: &mut R,
        prep: &KeygenPrep<P, N>,
    ) -> Option<(PublicKey, SecretKey, BigUint)> {
        let ed = prep.ed;
        let om = prep.om;
        let e = prep.e;
        let t = prep.t;
        let half = BigUint::one() << (e - 1);
        // 1. secret degree f, Kani scalars (c, c') with the canonical schedule
        let (f, c, cp) = loop {
            let f = &prep.lo + random_below(rng, &prep.lo);
            let f = if (&f % 2u32).is_zero() { f + 1u32 } else { f };
            if let Some((c, cp)) = Self::kani_scalars_for_degree(&f, false, rng) {
                break (f, c, cp);
            }
        };
        let g = &prep.n2 - &f;
        let fg = &f * &g;
        // 2. alpha = x0 + x1 omega + (y0 + y1 omega) pi with n(alpha) = f g
        let nform = |u0: &BigInt, u1: &BigInt| -> BigInt {
            u0 * u0 + BigInt::from(t) * u0 * u1 + u1 * u1 * 2
        };
        let ymax = (&fg / &prep.pchar).sqrt() + 1u32;
        let ymax_i = BigInt::from(ymax.clone());
        let mut alpha_c = None;
        for _ in 0..20000 {
            let y0 = BigInt::from(random_below(rng, &(&ymax * 2u32 + 1u32))) - &ymax_i;
            let y1 = BigInt::from(random_below(rng, &(&ymax * 2u32 + 1u32))) - &ymax_i;
            if y0.is_zero() && y1.is_zero() {
                continue;
            }
            let tt = BigInt::from(fg.clone()) - BigInt::from(prep.pchar.clone()) * nform(&y0, &y1);
            if tt <= BigInt::zero() {
                continue;
            }
            let tt = tt.to_biguint().unwrap();
            let r7 = (&tt % 7u32).to_u64_digits().first().copied().unwrap_or(0);
            if ![1u64, 2, 4].contains(&r7) || !is_probable_prime(&tt, rng, 0) {
                continue;
            }
            // x^2 + 7 y^2 = T  ->  x0 + x1 omega = (x - t y) + 2 y omega
            if let Some((xx, yy)) = cornacchia(7, &tt, rng) {
                let (xx, yy) = (BigInt::from(xx), BigInt::from(yy));
                alpha_c = Some((&xx - BigInt::from(t) * &yy, &yy * 2, y0, y1));
                break;
            }
        }
        let (x0, x1, y0, y1) = alpha_c?;
        let frob = |p: &WPoint<F<P, N>>| WPoint {
            x: p.x.frob(),
            y: p.y.frob(),
            inf: p.inf,
        };
        let mulz = |p: &WPoint<F<P, N>>, k: &BigInt| {
            let r = mul_vartime(&ed, p, &k.magnitude().clone());
            if k.sign() == num_bigint::Sign::Minus {
                r.neg()
            } else {
                r
            }
        };
        let alpha = |p: &WPoint<F<P, N>>| {
            let pp = frob(p);
            let a = add_vartime(&ed, &mulz(p, &x0), &mulz(&om.eval(p), &x1));
            let b = add_vartime(&ed, &mulz(&pp, &y0), &mulz(&om.eval(&pp), &y1));
            add_vartime(&ed, &a, &b)
        };
        // 3. Kani: kernel {([f] R, alpha R)}.
        // When p+1 has two spare bits of 2-torsion, sample the basis at order
        // 2^{e+2} and evaluate the (2^e, 2^e)-isogeny in the theta model.
        let fz = BigInt::from(f.clone());
        let (r1, r2) = torsion_basis_cof(&ed, 2, prep.level, &prep.cof2, rng);
        let r1e = if prep.use_theta {
            ed.dbl_iter(&r1, 2)
        } else {
            r1
        };
        let r2e = if prep.use_theta {
            ed.dbl_iter(&r2, 2)
        } else {
            r2
        };
        let w = |x: &WPoint<F<P, N>>| om.eval(x);
        let pd_e = Self::project_ev(&ed, &om, &r1e, &prep.nu, &half)
            .or_else(|| Self::project_ev(&ed, &om, &r2e, &prep.nu, &half))
            .unwrap_or_else(|| eigenvector_2k(&ed, &w, &prep.nu, e, rng));
        let qd_e = Self::project_ev(&ed, &om, &r1e, &prep.mu, &half)
            .or_else(|| Self::project_ev(&ed, &om, &r2e, &prep.mu, &half))
            .unwrap_or_else(|| eigenvector_2k(&ed, &w, &prep.mu, e, rng));
        let k1 = (mulz(&r1, &fz), alpha(&r1));
        let k2 = (mulz(&r2, &fz), alpha(&r2));
        let (g1, g2) = if prep.use_theta {
            (
                (ed.dbl_iter(&k1.0, 2), ed.dbl_iter(&k1.1, 2)),
                (ed.dbl_iter(&k2.0, 2), ed.dbl_iter(&k2.1, 2)),
            )
        } else {
            (k1, k2)
        };
        if !first_step_glues(&ed, &g1, &g2, e) {
            return None;
        }
        let (y3a_full, y3b_full) = torsion_basis_cof(&ed, 3, P::F3, &prep.cof3, rng);
        let r3 = y3a_full;
        let y3a = if prep.pair_cof.is_one() {
            y3a_full
        } else {
            mul_vartime(&ed, &y3a_full, &prep.pair_cof)
        };
        let y3b = if prep.pair_cof.is_one() {
            y3b_full
        } else {
            mul_vartime(&ed, &y3b_full, &prep.pair_cof)
        };
        let o = WPoint::infinity();
        let payload = [
            (add_vartime(&ed, &pd_e, &r3), o),
            (add_vartime(&ed, &qd_e, &r3), o),
            (r3, o),
            (y3a, o),
            (y3b, o),
        ];
        let (ea, eb, imgs) = if prep.use_theta {
            keygen_chain_theta(&ed, &k1, &k2, e, &payload)?
        } else {
            keygen_chain(&ed, &g1, &g2, e, &payload)
        };
        // 4. which factor carries eps (degree f)?  Short pairing of order 3^k.
        let base = weil_sq(&ed, &y3a, &y3b, &prep.pair_n);
        let want = base.pow_public(&(&f % &prep.pair_n).to_u64_digits());
        let inv_want = want.invert();
        let matches = |z: &F<P, N>| *z == want || *z == inv_want;
        let e1 = weil_sq(&ea, &imgs[3].0, &imgs[4].0, &prep.pair_n);
        let picked = if matches(&e1) {
            Some((ea, imgs[0].0, imgs[1].0, imgs[2].0))
        } else {
            let e2 = weil_sq(&eb, &imgs[3].1, &imgs[4].1, &prep.pair_n);
            if matches(&e2) {
                Some((eb, imgs[0].1, imgs[1].1, imgs[2].1))
            } else {
                None
            }
        };
        let (curve, a0, a1, a2) = picked?;
        let (eps_p, eps_q, t1, t2) = Self::recover_eigen(&curve, a0, a1, a2, e)?;
        // 5. Montgomery model (B = 1, correct twist) of E_A
        let roots = [t1.x, t2.x, -curve.a2 - t1.x - t2.x];
        let mut model = None;
        for r in roots {
            let ap = r.mul_small(3) + curve.a2;
            let bp = (r.mul_small(3) + curve.a2.double()) * r + curve.a4;
            let (s, ok) = bp.sqrt();
            if !bool::from(ok) {
                continue;
            }
            let (_, sok) = s.sqrt();
            if !bool::from(sok) {
                continue;
            }
            let si = s.invert();
            let a_m = ap * si;
            let wm = WCurve::from_montgomery(a_m);
            let rp = crate::cm::random_point(&wm, rng);
            if bool::from(mul_vartime(&wm, &rp, &(&prep.pchar + 1u32)).inf) {
                model = Some((r, si, a_m));
                break;
            }
        }
        let (r0, si, a_m) = model?;
        // 6. random diagonal rescaling of the eigenbasis
        let l1 = random_below(rng, &prep.n2) | BigUint::one();
        let l2 = random_below(rng, &prep.n2) | BigUint::one();
        let pp = mul_vartime(&curve, &eps_p, &l1);
        let qq = mul_vartime(&curve, &eps_q, &l2);
        let pq = curve.sub(&pp, &qq);
        let to_m = |x: F<P, N>| (x - r0) * si;
        let pk_f = [a_m, to_m(pp.x), to_m(qq.x), to_m(pq.x)];
        let pk_bytes = fbytes(&pk_f);
        // 7. secret key: a function of f, (c, c') and the public key
        let mut z = [0u8; 32];
        rng.fill_bytes(&mut z);
        let sk = Self::secret_key_from_parts(&pk_bytes, &f, &c, &cp, false, z);
        let pk = PublicKey { bytes: pk_bytes };
        // Variable-time correctness check (keygen only). A key that fails one
        // honest encaps/decaps round trip is rejected and the attempt retried.
        let (ct, mut ss_enc) = Self::encaps(&pk, rng)?;
        let mut ss_dec = Self::decaps(&sk, &ct);
        let ok = ss_enc == ss_dec;
        ss_enc.zeroize();
        ss_dec.zeroize();
        if !ok {
            return None;
        }
        Some((pk, sk, f))
    }
}

type EigenRec<P, const N: usize> = (
    WPoint<F<P, N>>,
    WPoint<F<P, N>>,
    WPoint<F<P, N>>,
    WPoint<F<P, N>>,
);

struct KeygenPrep<P: ParamSet<N>, const N: usize> {
    ed: WCurve<F<P, N>>,
    om: Omega<F<P, N>>,
    t: i64,
    pchar: BigUint,
    mu: BigUint,
    nu: BigUint,
    n2: BigUint,
    e: usize,
    use_theta: bool,
    level: usize,
    cof2: BigUint,
    cof3: BigUint,
    pair_cof: BigUint,
    pair_n: BigUint,
    lo: BigUint,
}

pub(crate) fn _unused(_: MontCurve<Fp2<crate::params::L1Fp, 21>>) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cm::point_of_order;
    use crate::params::L1;
    use rand_core::OsRng;

    #[test]
    fn scalar_recovery_handles_minus_one_and_zero() {
        let a = F::<L1, 21>::zero();
        let e = WCurve::from_montgomery(a);
        let mut rng = OsRng;
        for (l, exponent) in [(2u32, L1::E), (3u32, L1::F3)] {
            let p = point_of_order(&e, l, exponent, &mut rng);
            let order = BigUint::from(l).pow(exponent as u32);
            let bits = order.bits() as usize;
            let width = bits.div_ceil(64) + 1;
            let minus_one = Scalar::from_biguint(&(&order - 1u32), width);
            let got = smul_ct(&a, &p, &minus_one, bits);
            assert!(bool::from(got.ct_eq(&p.neg())), "order {l}^{exponent}");
            let zero = smul_ct(&a, &p, &Scalar::zero(width), bits);
            assert!(bool::from(zero.inf));
            let one = smul_ct(&a, &p, &Scalar::from_u64(1, width), bits);
            assert!(bool::from(one.ct_eq(&p)));
        }
    }

    #[test]
    fn rejects_invalid_public_curves_and_basis() {
        let mut rng = OsRng;
        let zero = PublicKey {
            bytes: vec![0; EigenSeta::<L1, 21>::pk_bytes()],
        };
        assert!(EigenSeta::<L1, 21>::encaps(&zero, &mut rng).is_none());
        let singular = PublicKey {
            bytes: fbytes(&[
                F::<L1, 21>::from_i64(2),
                F::<L1, 21>::zero(),
                F::<L1, 21>::zero(),
                F::<L1, 21>::zero(),
            ]),
        };
        assert!(EigenSeta::<L1, 21>::encaps(&singular, &mut rng).is_none());
    }

    #[test]
    fn theta_decaps_round_trips_l1() {
        use crate::drbg::Drbg;
        let mut rng = Drbg::for_vector("theta-kem-roundtrip", 0);
        let (pk, sk) = EigenSeta::<L1, 21>::keygen(&mut rng);
        let (ct, ss) = EigenSeta::<L1, 21>::encaps(&pk, &mut rng).unwrap();
        assert_eq!(EigenSeta::<L1, 21>::decaps(&sk, &ct), ss);
    }

    #[test]
    fn parallel_keygen_jobs_l1() {
        let (pk, sk) = EigenSeta::<L1, 21>::keygen_with_jobs(2);
        let mut rng = OsRng;
        let (ct, ss) = EigenSeta::<L1, 21>::encaps(&pk, &mut rng).unwrap();
        assert_eq!(EigenSeta::<L1, 21>::decaps(&sk, &ct), ss);
    }

    /// Probe δ_corr: several kernel scalars, including 0 and d−1, round-trip
    /// through the theta decapsulation path. Not a proof that δ_corr = 0.
    #[test]
    fn l1_recovers_several_kernel_scalars() {
        use crate::drbg::Drbg;
        let mut rng = Drbg::for_vector("delta-corr-l1", 0);
        let (pk, sk) = EigenSeta::<L1, 21>::keygen(&mut rng);
        let d = BigUint::from(3u32).pow(L1::F3 as u32);
        let mut scalars = vec![
            BigUint::from(0u32),
            BigUint::from(1u32),
            BigUint::from(2u32),
            BigUint::from(7u32),
            BigUint::from(1000u32),
            &d - 1u32,
        ];
        scalars.push(d.clone() / 2u32);
        for s in scalars {
            let ct = EigenSeta::<L1, 21>::encrypt_scalar(&pk, &s).unwrap();
            let got = EigenSeta::<L1, 21>::recover_s(&sk, &ct).expect("recover");
            assert_eq!(EigenSeta::<L1, 21>::scalar_to_biguint(&got) % &d, s % &d);
            let k1 = EigenSeta::<L1, 21>::decaps(&sk, &ct);
            let k2 = EigenSeta::<L1, 21>::decaps(&sk, &ct);
            assert_eq!(k1, k2);
        }
    }
}
