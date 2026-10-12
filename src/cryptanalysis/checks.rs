//! Structural conditions on a key that known attacks on related schemes
//! would need, checked on the actual secret degree.
//!
//! * `f` odd and coprime to 21: the discriminant of the hidden
//!   endomorphism θ' = 2θ ∓ f is Δ' = 7 f².  The self-pairing attack of
//!   Castryck–Houben–Merz–Mula–van Buuren–Vercauteren (CRYPTO 2023) needs
//!   an m > 1 with m | 2^E and m² | Δ', impossible when f is odd.
//!   Coprimality to 3 keeps the 3^F3-torsion computations clean and to 7
//!   keeps Δ' exactly one power of 7.
//! * `f` in the public range [lo, 2 lo): the attacker's search space is
//!   exactly what the cost model assumes.
//! * The eigenvalues of ω modulo 2 are distinct (√−7 is odd modulo 2^E),
//!   so the two public eigenlines span E_A[2^E]; and the Kani scalars for f
//!   exist, i.e. 2^E − 7 d² f² is a sum of two squares with the schedule's
//!   sign pattern (this is what key generation enforces).

use crate::kem::EigenSeta;
use crate::params::ParamSet;
use num_bigint::BigUint;
use num_integer::Integer;
use rand_core::{CryptoRng, RngCore};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyChecks {
    pub f_odd: bool,
    pub f_coprime_21: bool,
    pub f_in_range: bool,
    pub kani_scalars_exist: bool,
    /// The quantity Δ' = 7 f² is odd, which excludes the self-pairing
    /// attack for every 2-power torsion order.
    pub disc_odd: bool,
    pub f_bits: u64,
}

impl KeyChecks {
    pub fn all_pass(&self) -> bool {
        self.f_odd
            && self.f_coprime_21
            && self.f_in_range
            && self.kani_scalars_exist
            && self.disc_odd
    }
}

pub fn check_degree<P: ParamSet<N>, const N: usize, R: RngCore + CryptoRng>(
    f: &BigUint,
    rng: &mut R,
) -> KeyChecks {
    let (lo, hi) = EigenSeta::<P, N>::secret_degree_range();
    let f_odd = f.is_odd();
    let disc = f * f * 7u32;
    KeyChecks {
        f_odd,
        f_coprime_21: f.gcd(&BigUint::from(21u32)) == BigUint::from(1u32),
        f_in_range: f >= &lo && f < &hi,
        kani_scalars_exist: EigenSeta::<P, N>::kani_scalars_for_degree(f, false, rng).is_some(),
        disc_odd: disc.is_odd(),
        f_bits: f.bits(),
    }
}
