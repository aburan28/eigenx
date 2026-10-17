//! Confirm each parameter set's prime is the nothing-up-my-sleeve prime:
//! smallest cofactor c with p = 2^(E+1)·3^F3·c − 1 NUMS-prime and
//! p ≡ 3, 5 or 6 (mod 7). See docs/NUMS.md.
//!
//!   cargo run --release --example verify_nums -- L1
use eigen_seta::fp::FpConfig;
use eigen_seta::ntheory::is_nums_prime;
use eigen_seta::params::{ParamSet, L1, L3, L5, Q3, Q5};
use num_bigint::BigUint;

fn check<P: ParamSet<N>, const N: usize>() {
    let e = P::E;
    let f3 = P::F3;
    let c = P::COFACTOR;
    let base = (BigUint::from(1u32) << (e + 1)) * BigUint::from(3u32).pow(f3 as u32);
    let p = &base * c - 1u32;
    let modulus = BigUint::from_bytes_le(
        &P::Cfg::MODULUS
            .iter()
            .flat_map(|w| w.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    assert_eq!(p, modulus, "{} modulus mismatch", P::NAME);
    assert!(is_nums_prime(&p), "{} prime failed NUMS test", P::NAME);
    let mut smaller_congruent = 0u64;
    for c2 in 1..c {
        let q = &base * c2 - 1u32;
        let r7 = (&q % 7u32).to_u32_digits().first().copied().unwrap_or(0);
        if ![3, 5, 6].contains(&r7) {
            continue;
        }
        smaller_congruent += 1;
        assert!(
            !is_nums_prime(&q),
            "{}: smaller cofactor {c2} is also NUMS-prime",
            P::NAME
        );
    }
    println!(
        "{}: c = {c} is minimal ({smaller_congruent} smaller c with p ≡ 3,5,6 mod 7 rejected)",
        P::NAME
    );
}

fn main() {
    let want: Vec<String> = std::env::args().skip(1).collect();
    let all = want.is_empty();
    let go = |n: &str| all || want.iter().any(|w| w == n);
    if go("L1") {
        check::<L1, 21>();
    }
    if go("L3") {
        check::<L3, 32>();
    }
    if go("L5") {
        check::<L5, 42>();
    }
    if go("Q3") {
        check::<Q3, 61>();
    }
    if go("Q5") {
        check::<Q5, 82>();
    }
}
