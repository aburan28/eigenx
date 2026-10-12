//! Key recovery through the secret degree f.
//!
//! The secret key of Eigen-Séta v2 is determined by f and public data (see
//! `EigenSeta::secret_keys_from_degree`), so key recovery reduces to finding
//! f.  Two questions matter for the cost model:
//!
//! 1. Given the true f, does the derivation actually reproduce a working
//!    key?  (`recover_key_from_degree`.)  If it did not, the key would
//!    contain more secret entropy than f and the search cost would be
//!    higher; the cost model assumes the pessimistic answer "yes".
//! 2. Does a wrong guess f' that agrees with f modulo 2^k behave any
//!    differently from a random wrong guess?  If the decapsulation chain
//!    failed "earlier" for distant guesses and "later" for close ones, an
//!    attacker could recover f bit by bit in O(E) chain evaluations instead
//!    of 2^(f_bits).  (`near_miss_experiment`.)  The design note lists this
//!    as unverified; the experiment measures it.

use crate::chain::ChainReport;
use crate::kem::{Ciphertext, EigenSeta, PublicKey, SecretKey};
use crate::ntheory::random_below;
use crate::params::ParamSet;
use num_bigint::BigUint;
use num_traits::One;
use rand_core::{CryptoRng, RngCore};

/// A fresh honest ciphertext and its key under `pk`.
fn probe<P: ParamSet<N>, const N: usize, R: RngCore + CryptoRng>(
    pk: &PublicKey,
    rng: &mut R,
) -> (Ciphertext, [u8; 32]) {
    EigenSeta::<P, N>::encaps(pk, rng).expect("honest public key")
}

/// Try to recover a working secret key for `pk` from a candidate degree.
/// Returns the first candidate that decapsulates a fresh ciphertext to the
/// right key.  Variable time.
pub fn recover_key_from_degree<P: ParamSet<N>, const N: usize, R: RngCore + CryptoRng>(
    pk: &PublicKey,
    f: &BigUint,
    rng: &mut R,
) -> Option<SecretKey> {
    let (ct, k) = probe::<P, N, R>(pk, rng);
    EigenSeta::<P, N>::secret_keys_from_degree(pk, f, rng)
        .into_iter()
        .find(|sk| EigenSeta::<P, N>::decaps(sk, &ct) == k)
}

/// Exhaustive search of f over `[lo, hi)` (odd values only).  Only sensible
/// for tiny ranges; it exists to make the per-guess cost measurable.
/// Returns the degree, the key, and the number of candidates tested.
pub fn exhaustive_degree_search<P: ParamSet<N>, const N: usize, R: RngCore + CryptoRng>(
    pk: &PublicKey,
    lo: &BigUint,
    hi: &BigUint,
    rng: &mut R,
) -> Option<(BigUint, SecretKey, u64)> {
    let (ct, k) = probe::<P, N, R>(pk, rng);
    let mut f = lo.clone();
    if (&f % 2u32) == BigUint::default() {
        f += 1u32;
    }
    let mut tested = 0u64;
    while &f < hi {
        for sk in EigenSeta::<P, N>::secret_keys_from_degree(pk, &f, rng) {
            tested += 1;
            if EigenSeta::<P, N>::decaps(&sk, &ct) == k {
                return Some((f, sk, tested));
            }
        }
        f += 2u32;
    }
    None
}

/// One row of the near-miss experiment.
#[derive(Clone, Debug)]
pub struct NearMissRow {
    /// Number of low bits in which the guess agrees with the true f
    /// (`None` for the true f itself).
    pub agree_bits: Option<usize>,
    /// Decapsulation with the derived key returned the encapsulated key.
    pub decaps_correct: bool,
    /// Variable-time walk of the chain with structural checks.
    pub report: Option<ChainReport>,
}

/// The `t`-th value f' in [lo, hi), odd, with f' ≡ f (mod 2^k) and f' ≠ f,
/// together with the number of such slots.  `t` is taken modulo the slot
/// count; returns `None` for the slot holding f itself or if the range
/// leaves no room (k too large).
fn near_miss_candidate(
    f: &BigUint,
    k: usize,
    lo: &BigUint,
    hi: &BigUint,
    t: &BigUint,
) -> (Option<BigUint>, BigUint) {
    let step = BigUint::one() << k.max(1); // keep f' odd
    let span = hi - lo;
    if span <= step {
        return (None, BigUint::default());
    }
    let slots = &span / &step;
    let base = lo - (lo % &step) + (f % &step);
    let cand = &base + (t % &slots) * &step;
    if &cand >= lo && &cand < hi && &cand != f {
        (Some(cand), slots)
    } else {
        (None, slots)
    }
}

/// For each k in `agree_bits`, derive keys from `trials` wrong degrees that
/// agree with the true f on the low k bits, test them, and walk the chain
/// with checks.  The row with `agree_bits = None` is the true f.
pub fn near_miss_experiment<P: ParamSet<N>, const N: usize, R: RngCore + CryptoRng>(
    pk: &PublicKey,
    f: &BigUint,
    agree_bits: &[usize],
    trials: usize,
    rng: &mut R,
) -> Vec<NearMissRow> {
    let (lo, hi) = EigenSeta::<P, N>::secret_degree_range();
    let (ct, k) = probe::<P, N, R>(pk, rng);
    let mut rows = vec![];
    // The true degree: the honest (unflipped) candidate must work and the
    // chain must split exactly where scheduled.
    for sk in EigenSeta::<P, N>::secret_keys_from_degree(pk, f, rng) {
        rows.push(NearMissRow {
            agree_bits: None,
            decaps_correct: EigenSeta::<P, N>::decaps(&sk, &ct) == k,
            report: EigenSeta::<P, N>::inspect_decaps_chain(&sk, &ct),
        });
    }
    for &kb in agree_bits {
        let mut done = 0;
        let mut attempts = 0u64;
        // Only a small fraction of candidate degrees admit Kani scalars
        // (2^E - 7 d^2 f'^2 must be a sum of two squares with the right
        // 2-adic signs), exactly as in honest key generation, so allow a
        // generous number of cheap failures per usable sample.  When the
        // residue class is small enough we walk it exhaustively from a random
        // offset instead of sampling with replacement.
        let budget = 4000 * trials as u64;
        let (_, slots) = near_miss_candidate(f, kb, &lo, &hi, &BigUint::default());
        if slots == BigUint::default() {
            continue;
        }
        let exhaustive = slots <= BigUint::from(budget);
        let offset = if exhaustive {
            random_below(rng, &slots)
        } else {
            BigUint::default()
        };
        let cap = if exhaustive {
            u64::try_from(&slots).unwrap_or(budget).min(budget)
        } else {
            budget
        };
        while done < trials && attempts < cap {
            let t = if exhaustive {
                &offset + attempts
            } else {
                random_below(rng, &slots)
            };
            attempts += 1;
            let (Some(fp), _) = near_miss_candidate(f, kb, &lo, &hi, &t) else {
                continue;
            };
            for sk in EigenSeta::<P, N>::secret_keys_from_degree(pk, &fp, rng) {
                rows.push(NearMissRow {
                    agree_bits: Some(kb),
                    decaps_correct: EigenSeta::<P, N>::decaps(&sk, &ct) == k,
                    report: EigenSeta::<P, N>::inspect_decaps_chain(&sk, &ct),
                });
                done += 1;
            }
        }
    }
    rows
}

/// Summary statistics of a near-miss experiment, per agreement level.
#[derive(Clone, Debug, Default)]
pub struct NearMissSummary {
    pub rows: usize,
    pub decaps_correct: usize,
    pub prodker_ok: usize,
    pub glue_ok: usize,
    pub early_split: usize,
    pub final_split: usize,
    pub mean_richelot_steps: f64,
}

pub fn summarise(rows: &[NearMissRow], agree_bits: Option<usize>) -> NearMissSummary {
    let mut s = NearMissSummary::default();
    let mut steps = 0usize;
    for r in rows.iter().filter(|r| r.agree_bits == agree_bits) {
        s.rows += 1;
        s.decaps_correct += r.decaps_correct as usize;
        if let Some(rep) = &r.report {
            s.prodker_ok += rep.prodker_ok as usize;
            s.glue_ok += rep.glue_ok as usize;
            s.early_split += rep.first_zero_delta_step.is_some() as usize;
            s.final_split += rep.final_split as usize;
            steps += rep.richelot_steps;
        }
    }
    if s.rows > 0 {
        s.mean_richelot_steps = steps as f64 / s.rows as f64;
    }
    s
}
