//! Mask-guess + Kani message recovery (FESTA CIST / Robert dim-4).
//!
//! Given a ciphertext, the Weil pairing of the masked images leaks
//! `a·b·d mod 2^E`. An attacker who guesses the masks modulo `2^{e_mask}`
//! with `2^e = d + a1² + a2²` (Robert / KaniEndo) can build the kernel of
//! the endomorphism
//!
//! ```text
//! F = [[α,  diag(ˆφ)],
//!      [-diag(φ), ˆα]]  ∈ End(E_A² × E_C²)
//! ```
//!
//! with `α = [[a1,a2],[-a2,a1]]` and recover φ (hence the sender scalar s)
//! by evaluating F. Full evaluation needs dimension-4 theta isogenies
//! (ePrint 2024/1180, Theta_dim4). This module:
//!
//! 1. finds the Robert parameters `(e, a1, a2)` for the set;
//! 2. enumerates candidate masks (or tries a known honest pair);
//! 3. unmasks the claimed torsion images;
//! 4. builds the `form_kernel` generators of Robert ePrint 2022/1038 §4;
//! 5. times one `product_isogeny` of length `e_mask` as the cost-model
//!    oracle (the dim-2 steps present in every dim-4 Kani chain);
//! 6. recovers `s` on a correct mask via the existing MITM (the poly-time
//!    dim-4 evaluation that replaces MITM once masks are known lives in
//!    `research/kani4_sage/`).
//!
//! CI covers (5) and the correct-mask recovery; the Sage driver covers the
//! full dim-4 evaluation against Theta_dim4.

use crate::cm::{mul_vartime, point_from_x};
use crate::field::Field;
use crate::fp2::Fp2;
use crate::kem::{Ciphertext, EigenSeta, PublicKey};
use crate::mont::{ladder, ProjCurve, XPoint};
use crate::ntheory::two_squares;
use crate::params::ParamSet;
use crate::scalar::Scalar;
use crate::strategy::optimal;
use crate::theta::{self, CouplePoint, EllipticProduct};
use crate::weier::{WCurve, WPoint};
use num_bigint::BigUint;
use num_traits::One;
use rand_core::{CryptoRng, RngCore};
use std::time::Instant;

type Fq<P, const N: usize> = Fp2<<P as ParamSet<N>>::Cfg, N>;

/// Result of one mask-guess Kani experiment.
#[derive(Clone, Debug)]
pub struct Kani4Result {
    pub recovered_s: Option<BigUint>,
    pub confirmed_by_reencryption: bool,
    pub e: usize,
    pub e_mask: usize,
    pub a1: BigUint,
    pub a2: BigUint,
    pub guesses_tried: u64,
    pub seconds: f64,
    pub oracle_seconds: f64,
    pub note: &'static str,
}

/// Robert degree equation: find the smallest `e` with
/// `2^e = d + a1² + a2²` and `a1` odd (KaniEndo convention).
pub fn robert_params<P: ParamSet<N>, const N: usize, R: RngCore>(
    rng: &mut R,
) -> Option<(usize, BigUint, BigUint)> {
    let d = BigUint::from(3u32).pow(P::F3 as u32);
    let lo = (P::F3 as f64 * 3f64.log2()).ceil() as usize;
    for e in lo..=P::E.min(lo + 64) {
        let two_e = BigUint::one() << e;
        if two_e <= d {
            continue;
        }
        let m = &two_e - &d;
        if let Some((x, y)) = two_squares(&m, rng, 1 << 16) {
            let (a1, a2) = if (&x % 2u32).is_one() {
                (x, y)
            } else if (&y % 2u32).is_one() {
                (y, x)
            } else {
                continue;
            };
            return Some((e, a1, a2));
        }
    }
    None
}

/// Mask precision for the half-chain attack: `⌈e/2⌉ + 2` (KaniEndoHalf).
pub fn e_mask_for(e: usize) -> usize {
    e.div_ceil(2) + 2
}

fn parse4<P: ParamSet<N>, const N: usize>(bytes: &[u8]) -> Option<[Fq<P, N>; 4]> {
    let l = <Fq<P, N> as Field>::byte_len();
    if bytes.len() != 4 * l {
        return None;
    }
    let mut out = [<Fq<P, N> as Field>::zero(); 4];
    for i in 0..4 {
        let (v, ok) = <Fq<P, N> as Field>::from_bytes(&bytes[i * l..(i + 1) * l]);
        if !bool::from(ok) {
            return None;
        }
        out[i] = v;
    }
    Some(out)
}

/// Unmask the ciphertext's torsion images with candidate `(a, b)` mod `2^{e_mask}`.
#[allow(clippy::type_complexity)]
pub fn unmask<P: ParamSet<N>, const N: usize>(
    ct: &Ciphertext,
    a: &Scalar,
    b: &Scalar,
    e_mask: usize,
) -> Option<(WPoint<Fq<P, N>>, WPoint<Fq<P, N>>)> {
    let [ac, xs, xt, _] = parse4::<P, N>(&ct.bytes)?;
    let curve = WCurve::from_montgomery(ac);
    let pc = ProjCurve::from_affine(ac);
    // Full-width inverse mod 2^E (masks live mod 2^E); ladder length e_mask.
    let a_inv = a.inv_mod_2k(P::E);
    let b_inv = b.inv_mod_2k(P::E);
    let (xp, _) = ladder(&XPoint::from_affine(xs), &a_inv, e_mask, &pc);
    let (xq, _) = ladder(&XPoint::from_affine(xt), &b_inv, e_mask, &pc);
    let p = point_from_x(&curve, xp.affine())?;
    let q = point_from_x(&curve, xq.affine())?;
    Some((p, q))
}

/// One generator of Robert's kernel on E1² × E2².
#[derive(Clone, Debug)]
pub struct KernelGen<F: crate::field::Field> {
    pub e1: [WPoint<F>; 2],
    pub e2: [WPoint<F>; 2],
}

/// `form_kernel` from Theta_dim4 `Kani_endomorphism.py` (Robert §4).
pub fn form_kernel<F: crate::field::Field>(
    a1: &BigUint,
    a2: &BigUint,
    p1: &WPoint<F>,
    q1: &WPoint<F>,
    r2: &WPoint<F>,
    s2: &WPoint<F>,
    e1: &WCurve<F>,
    _e2: &WCurve<F>,
    e_kani: usize,
) -> [KernelGen<F>; 4] {
    let x_p = mul_vartime(e1, p1, a1);
    let y_p = mul_vartime(e1, p1, a2);
    let x_q = mul_vartime(e1, q1, a1);
    let y_q = mul_vartime(e1, q1, a2);
    let t1 = e1.dbl_iter(p1, e_kani);
    let u1 = e1.dbl_iter(q1, e_kani);
    let o2 = WPoint::infinity();
    let a1_mod4 = (a1 % 4u32).to_u32_digits().first().copied().unwrap_or(0);
    if a1_mod4 == 1 || a1_mod4 == 3 {
        let a = if a1_mod4 == 1 {
            BigUint::from(3u32)
        } else {
            BigUint::one()
        };
        [
            KernelGen {
                e1: [e1.add(&x_p, &mul_vartime(e1, &t1, &a)), y_p],
                e2: [*r2, o2],
            },
            KernelGen {
                e1: [x_q, y_q],
                e2: [*s2, o2],
            },
            KernelGen {
                e1: [y_p.neg(), x_p],
                e2: [o2, *r2],
            },
            KernelGen {
                e1: [y_q.neg(), e1.add(&x_q, &mul_vartime(e1, &u1, &a))],
                e2: [o2, *s2],
            },
        ]
    } else {
        let a2_mod4 = (a2 % 4u32).to_u32_digits().first().copied().unwrap_or(0);
        let b = if a2_mod4 == 1 {
            BigUint::from(3u32)
        } else {
            BigUint::one()
        };
        [
            KernelGen {
                e1: [x_p, e1.add(&y_p, &mul_vartime(e1, &t1, &b))],
                e2: [*r2, o2],
            },
            KernelGen {
                e1: [x_q, y_q],
                e2: [*s2, o2],
            },
            KernelGen {
                e1: [y_p.neg(), x_p],
                e2: [o2, *r2],
            },
            KernelGen {
                e1: [e1.add(&y_q.neg(), &mul_vartime(e1, &u1, &b).neg()), x_q],
                e2: [o2, *s2],
            },
        ]
    }
}

/// Cost-model oracle: one `(2^n, 2^n)`-isogeny between elliptic products.
pub fn oracle_product_isogeny<F: crate::field::Field + Send + Sync>(
    e_a: &WCurve<F>,
    e_c: &WCurve<F>,
    p_a: &WPoint<F>,
    q_a: &WPoint<F>,
    p_c: &WPoint<F>,
    q_c: &WPoint<F>,
    n: usize,
) -> bool {
    if n < 3 {
        return false;
    }
    let prod: EllipticProduct<F> = (*e_a, *e_c);
    let k1: CouplePoint<F> = (*p_a, *p_c);
    let k2: CouplePoint<F> = (*q_a, *q_c);
    let strategy = optimal(n, 0.8, 1.0);
    let _ = theta::product_isogeny(&prod, &k1, &k2, &[], n, &strategy);
    true
}

fn scalar_from_u64(v: u64, limbs: usize) -> Scalar {
    let mut s = Scalar::zero(limbs);
    s.l[0] = v;
    s
}

/// Run the mask-guess attack scaffolding.
///
/// Without a dim-4 theta evaluator, a wrong mask cannot be rejected
/// algebraically here. The recoverable path is therefore:
///
/// * `known_masks = Some((a,b))` — the honest FO masks (from a known `s`);
///   one `form_kernel` + product-isogeny oracle step is timed, then `s` is
///   recovered by the existing MITM (stand-in for the poly-time dim-4
///   evaluation that lives in `research/kani4_sage/`).
/// * `known_masks = None` — enumerate up to `max_guesses` odd units, time
///   the oracle on each, and do **not** claim recovery (mask space is
///   2^{2·e_mask}; full search is the cost-model experiment, not CI).
pub fn attack<P: ParamSet<N>, const N: usize, R: RngCore + CryptoRng>(
    pk: &PublicKey,
    ct: &Ciphertext,
    max_guesses: Option<u64>,
    known_masks: Option<(Scalar, Scalar)>,
    rng: &mut R,
) -> Kani4Result {
    let t0 = Instant::now();
    let Some((e, a1, a2)) = robert_params::<P, N, _>(rng) else {
        return Kani4Result {
            recovered_s: None,
            confirmed_by_reencryption: false,
            e: 0,
            e_mask: 0,
            a1: BigUint::default(),
            a2: BigUint::default(),
            guesses_tried: 0,
            seconds: t0.elapsed().as_secs_f64(),
            oracle_seconds: 0.0,
            note: "no Robert parameters (2^e = d + a1^2 + a2^2)",
        };
    };
    let e_mask = e_mask_for(e).min(P::E);
    let limbs = P::E.div_ceil(64);

    let had_honest = known_masks.is_some();
    let mut candidates: Vec<(Scalar, Scalar, bool)> = Vec::new();
    if let Some((a, b)) = known_masks {
        candidates.push((a, b, true));
        for v in [3u64, 5, 7, 9, 11] {
            let mut aa = scalar_from_u64(v, limbs);
            aa.l[0] |= 1;
            let mut bb = scalar_from_u64(v.wrapping_mul(3), limbs);
            bb.l[0] |= 1;
            candidates.push((aa, bb, false));
        }
    } else {
        let limit = max_guesses.unwrap_or(64);
        for v in (1u64..).step_by(2).take(limit as usize) {
            let mut a = scalar_from_u64(v, limbs);
            a.l[0] |= 1;
            candidates.push((a.clone(), a, false));
        }
    }

    let Some([aa, xp, xq, _]) = parse4::<P, N>(&pk.bytes) else {
        return Kani4Result {
            recovered_s: None,
            confirmed_by_reencryption: false,
            e,
            e_mask,
            a1,
            a2,
            guesses_tried: 0,
            seconds: t0.elapsed().as_secs_f64(),
            oracle_seconds: 0.0,
            note: "bad public key",
        };
    };
    let Some([ac, _, _, _]) = parse4::<P, N>(&ct.bytes) else {
        return Kani4Result {
            recovered_s: None,
            confirmed_by_reencryption: false,
            e,
            e_mask,
            a1,
            a2,
            guesses_tried: 0,
            seconds: t0.elapsed().as_secs_f64(),
            oracle_seconds: 0.0,
            note: "bad ciphertext",
        };
    };
    let ea = WCurve::from_montgomery(aa);
    let ec = WCurve::from_montgomery(ac);
    let Some(p_full) = point_from_x(&ea, xp) else {
        return Kani4Result {
            recovered_s: None,
            confirmed_by_reencryption: false,
            e,
            e_mask,
            a1,
            a2,
            guesses_tried: 0,
            seconds: t0.elapsed().as_secs_f64(),
            oracle_seconds: 0.0,
            note: "pk basis not on curve",
        };
    };
    let Some(q_full) = point_from_x(&ea, xq) else {
        return Kani4Result {
            recovered_s: None,
            confirmed_by_reencryption: false,
            e,
            e_mask,
            a1,
            a2,
            guesses_tried: 0,
            seconds: t0.elapsed().as_secs_f64(),
            oracle_seconds: 0.0,
            note: "pk basis not on curve",
        };
    };
    let lift = P::E.saturating_sub(e_mask);
    let p_m = ea.dbl_iter(&p_full, lift);
    let q_m = ea.dbl_iter(&q_full, lift);

    let mut guesses = 0u64;
    let mut oracle_secs = 0.0;
    // Oracle length capped for CI time; cost model uses full e_mask.
    let oracle_n = e_mask.clamp(3, 12);

    for (a, b, is_honest) in &candidates {
        guesses += 1;
        let Some((phi_p, phi_q)) = unmask::<P, N>(ct, a, b, e_mask) else {
            continue;
        };
        let _ker = form_kernel(&a1, &a2, &p_m, &q_m, &phi_p, &phi_q, &ea, &ec, e);

        let t1 = Instant::now();
        let _ = oracle_product_isogeny(&ea, &ec, &p_m, &q_m, &phi_p, &phi_q, oracle_n);
        oracle_secs += t1.elapsed().as_secs_f64();

        if !*is_honest {
            continue;
        }
        // Honest mask: MITM stands in for dim-4 evaluation (Sage).
        let r = crate::cryptanalysis::mitm3::attack::<P, N>(pk, ct);
        if r.confirmed_by_reencryption {
            if let Some(s) = r.recovered_s.clone() {
                if let Some(ct2) = EigenSeta::<P, N>::encrypt_scalar(pk, &s) {
                    if ct2.bytes == ct.bytes {
                        return Kani4Result {
                            recovered_s: Some(s),
                            confirmed_by_reencryption: true,
                            e,
                            e_mask,
                            a1,
                            a2,
                            guesses_tried: guesses,
                            seconds: t0.elapsed().as_secs_f64(),
                            oracle_seconds: oracle_secs,
                            note: "recovered (honest mask + MITM stand-in; dim-4: research/kani4_sage)",
                        };
                    }
                }
            }
        }
    }

    Kani4Result {
        recovered_s: None,
        confirmed_by_reencryption: false,
        e,
        e_mask,
        a1,
        a2,
        guesses_tried: guesses,
        seconds: t0.elapsed().as_secs_f64(),
        oracle_seconds: oracle_secs,
        note: if had_honest {
            "honest mask tried but MITM did not confirm"
        } else {
            "oracle timed on guesses; no recovery without dim-4 / known masks"
        },
    }
}
