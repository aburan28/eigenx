//! The trial-division sieve in `is_probable_prime` changes only cost: it must
//! agree with plain Miller–Rabin (`is_nums_prime`) and with exact primality.
use eigen_seta::ntheory::{is_nums_prime, is_probable_prime};
use num_bigint::BigUint;
use rand_core::OsRng;

#[test]
fn sieve_agrees_with_exact_primality_below_2_18() {
    let lim = 1usize << 18;
    let mut comp = vec![false; lim];
    comp[0] = true;
    comp[1] = true;
    for i in 2..lim {
        if !comp[i] {
            for j in (i * i..lim).step_by(i) {
                comp[j] = true;
            }
        }
    }
    for (n, &c) in comp.iter().enumerate() {
        assert_eq!(
            is_probable_prime(&BigUint::from(n), &mut OsRng, 0),
            !c,
            "n = {n}"
        );
    }
}

#[test]
fn sieve_agrees_with_miller_rabin_on_large_odd_numbers() {
    // 1500 consecutive odd numbers above 2^255 and 2^521: every decision,
    // prime and composite, matches the unsieved test.
    for base in [BigUint::from(1u32) << 255, BigUint::from(1u32) << 521] {
        let mut primes = 0;
        for k in 0..1500u32 {
            let n = &base + 2u32 * k + 1u32;
            let a = is_probable_prime(&n, &mut OsRng, 0);
            assert_eq!(a, is_nums_prime(&n), "n = {n}");
            primes += a as usize;
        }
        assert!(primes > 0);
    }
}
