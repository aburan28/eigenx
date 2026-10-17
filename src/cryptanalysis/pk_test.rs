//! Testing a candidate secret degree on the public key alone.
//!
//! The cost model (`security::estimate`, row "secret degree search") prices
//! one test of a candidate f as one decapsulation: a (2^E, 2^E)-chain on
//! E_C × E_C plus the 3-isogeny re-encryption.  An attacker does not need a
//! ciphertext.  With a candidate f the action of ψ_A = 2τ − t f on the public
//! basis is known exactly (f·s_D on P_A, −f·s_D on Q_A), and ψ_A has degree
//! 7 f², not 7 d² f².  So for any odd u and any e with
//! m = 2^e − 7 u² f² a sum of two squares c² + c′², the endomorphism
//! γ = u ψ_A + c of E_A has degree 2^e − c′² and Kani's lemma gives a
//! (2^e, 2^e)-isogeny of E_A × E_A with kernel
//! ⟨(c′P′, −λ_S P′), (c′Q′, −λ_T Q′)⟩, where P′ = [2^(E−e)] P_A and
//! λ_S = u f s_D + c, λ_T = c − u f s_D.  The true f makes the chain split
//! back into E_A × E_A.  A wrong f′ still gives an isotropic kernel of the
//! right order, because its own scalars satisfy
//! c′² + λ_S λ_T ≡ c′² + c² + 7u²f′² = 2^e (mod 2^e), so the walk runs to
//! the end and only the final split fails, as for decapsulation.
//!
//! e only has to exceed log2(7 u² f²) ≈ 2·f_bits + 3, against E ≈ 4·f_bits
//! for decapsulation, so the per-candidate test is about half as long and
//! needs no ciphertext.  This module finds the shortest such plan and runs
//! it; `examples/pk_degree_test.rs` measures it.  Variable time.

use crate::chain::{decaps_chain_theta, inspect_chain, ChainReport, Pair};
use crate::cm::mul_vartime;
use crate::kem::{EigenSeta, PublicKey};
use crate::ntheory::{modpos, sqrt_mod_2power, two_squares, v2};
use crate::params::ParamSet;
use crate::weier::{WCurve, WPoint};
use num_bigint::{BigInt, BigUint};
use num_traits::{One, Zero};
use rand_core::RngCore;

/// Kani data for testing one candidate degree on E_A × E_A.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PkTestPlan {
    /// Chain length: the test is a (2^e, 2^e)-isogeny.
    pub e: usize,
    /// Odd multiplier of ψ_A in γ = u ψ_A + c.
    pub u: u64,
    /// The even Kani scalar c and the odd one c′, c² + c′² = 2^e − 7 u² f².
    pub c: BigInt,
    pub cp: BigInt,
    /// Which public basis point carries the eigenvalue +f·s_D.
    pub flip: bool,
}

/// The shortest plan for candidate `f`: the smallest e (then the smallest
/// odd u ≤ `max_u`) for which 2^e − 7u²f² splits as c² + c′² with c even and
/// a sign choice meeting the fixed schedule's 2-adic condition
/// max(v2(c′ − λ_S), v2(c′ + λ_S)) = 2, as in key generation.  `None` if no
/// e ≤ `e_max` works.
pub fn plan_for_degree<R: RngCore>(
    f: &BigUint,
    flip: bool,
    big_e: usize,
    e_max: usize,
    max_u: u64,
    rng: &mut R,
) -> Option<PkTestPlan> {
    // The 2-adic root of −7 used by the KEM (mod 2^(E+2), then reduced).
    let s_full = sqrt_mod_2power(&((BigUint::one() << (big_e + 2)) - 7u32), big_e as u64 + 2);
    let seven_f2 = f * f * 7u32;
    let e_min = (seven_f2.bits() as usize + 1).max(8);
    for e in e_min..=e_max.min(big_e) {
        let n2 = BigUint::one() << e;
        let s_d = &s_full % &n2;
        let s_d = if flip { (&n2 - &s_d) % &n2 } else { s_d };
        let mut u = 1u64;
        while u <= max_u {
            let m7 = &seven_f2 * u * u;
            if m7 >= n2 {
                break;
            }
            let m = &n2 - &m7;
            if let Some((x, y)) = two_squares(&m, rng, 1 << 14) {
                let (c0, cp0) = if (&x % 2u32).is_zero() {
                    (x, y)
                } else {
                    (y, x)
                };
                let us0 = BigInt::from((f * &s_d * u) % &n2);
                let mut choice = None;
                for sc in [1i32, -1] {
                    for scp in [1i32, -1] {
                        let c = BigInt::from(c0.clone()) * sc;
                        let cp = BigInt::from(cp0.clone()) * scp;
                        let lam_s = &us0 + &c;
                        let lam_t = &c - &us0;
                        let minus1 = |z: &BigInt| modpos(&(z + 1), &n2).is_zero();
                        if v2(&(&cp - &lam_s)).max(v2(&(&cp + &lam_s))) == 2
                            && !minus1(&lam_s)
                            && !minus1(&lam_t)
                            && !minus1(&cp)
                        {
                            choice = Some((c, cp));
                        }
                    }
                }
                if let Some((c, cp)) = choice {
                    return Some(PkTestPlan { e, u, c, cp, flip });
                }
            }
            u += 2;
        }
    }
    None
}

type Fq<P, const N: usize> = crate::fp2::Fp2<<P as ParamSet<N>>::Cfg, N>;
type TestKernel<P, const N: usize> = (WCurve<Fq<P, N>>, [Pair<Fq<P, N>>; 2]);

/// The kernel generators of the test isogeny on E_A × E_A for `plan`.
fn kernel<P: ParamSet<N>, const N: usize>(
    pk: &PublicKey,
    f: &BigUint,
    plan: &PkTestPlan,
) -> Option<TestKernel<P, N>> {
    let (a, pa, qa) = EigenSeta::<P, N>::public_basis_2(pk)?;
    let wa = WCurve::from_montgomery(a);
    let e = plan.e;
    let n2 = BigUint::one() << e;
    let s_full = sqrt_mod_2power(&((BigUint::one() << (P::E + 2)) - 7u32), P::E as u64 + 2);
    let s_d = &s_full % &n2;
    let s_d = if plan.flip { (&n2 - &s_d) % &n2 } else { s_d };
    let us0 = BigInt::from((f * &s_d * plan.u) % &n2);
    let lam_s = modpos(&(&us0 + &plan.c), &n2);
    let lam_t = modpos(&(&plan.c - &us0), &n2);
    let cp = modpos(&plan.cp, &n2);
    // P′, Q′ of order 2^e.
    let p1 = wa.dbl_iter(&pa, P::E - e);
    let q1 = wa.dbl_iter(&qa, P::E - e);
    let k1 = (
        mul_vartime(&wa, &p1, &cp),
        mul_vartime(&wa, &p1, &lam_s).neg(),
    );
    let k2 = (
        mul_vartime(&wa, &q1, &cp),
        mul_vartime(&wa, &q1, &lam_t).neg(),
    );
    Some((wa, [k1, k2]))
}

/// Walk the test chain with the structural checks of `chain::inspect_chain`.
/// The true degree gives `final_split = true`.
pub fn inspect_pk_test<P: ParamSet<N>, const N: usize>(
    pk: &PublicKey,
    f: &BigUint,
    plan: &PkTestPlan,
) -> Option<ChainReport> {
    let (wa, [k1, k2]) = kernel::<P, N>(pk, f, plan)?;
    Some(inspect_chain(&wa, &k1, &k2, plan.e))
}

/// Run the test chain in the theta model (the decapsulation chain code at
/// length e instead of E) and report whether its codomain is E_A, which it
/// is for the true degree.  Only call this with a candidate that
/// `inspect_pk_test` accepted: the closing isomorphism step assumes the
/// split happened.
pub fn run_pk_test_theta<P: ParamSet<N>, const N: usize>(
    pk: &PublicKey,
    f: &BigUint,
    plan: &PkTestPlan,
) -> Option<bool> {
    use crate::ct::ConstantTimeEq;
    let (wa, [k1, k2]) = kernel::<P, N>(pk, f, plan)?;
    let (a, _, _) = EigenSeta::<P, N>::public_basis_2(pk)?;
    let (x1, _) = EigenSeta::<P, N>::public_basis_3(&a)?;
    let (fc, _) = decaps_chain_theta(&wa, &k1, &k2, plan.e, &[(x1, WPoint::infinity())]);
    Some(bool::from(fc.j_invariant().ct_eq(&wa.j_invariant())))
}
