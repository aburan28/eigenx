//! Nothing-up-my-sleeve cofactors. L1 is checked here; the other sets are
//! the same test via `cargo run --release --example verify_nums`.
use eigen_seta::fp::FpConfig;
use eigen_seta::ntheory::{is_nums_prime, NUMS_MR_BASES};
use eigen_seta::params::{L1, ParamSet};
use num_bigint::BigUint;

fn minimal<P: ParamSet<N>, const N: usize>() {
    let base = (BigUint::from(1u32) << (P::E + 1)) * BigUint::from(3u32).pow(P::F3 as u32);
    let p = &base * P::COFACTOR - 1u32;
    let modulus = BigUint::from_bytes_le(
        &P::Cfg::MODULUS
            .iter()
            .flat_map(|w| w.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    assert_eq!(p, modulus);
    assert!(is_nums_prime(&p));
    for c2 in 1..P::COFACTOR {
        let q = &base * c2 - 1u32;
        let r7 = (&q % 7u32).to_u32_digits().first().copied().unwrap_or(0);
        if [3u32, 5, 6].contains(&r7) {
            assert!(!is_nums_prime(&q), "c={c2}");
        }
    }
}

#[test]
fn nums_bases_are_public_and_fixed() {
    assert_eq!(NUMS_MR_BASES[0], 2);
    assert_eq!(*NUMS_MR_BASES.last().unwrap(), 37);
    assert!(!is_nums_prime(&BigUint::from(91u32)));
    assert!(is_nums_prime(&BigUint::from(97u32)));
}

#[test]
fn l1_cofactor_is_minimal() {
    minimal::<L1, 21>();
}
