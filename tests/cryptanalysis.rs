//! Toy-scale cryptanalysis experiments that back the cost model in
//! `docs/SECURITY.md`.  Larger runs: `cargo run --release --example
//! cryptanalysis`.

use eigen_seta::cryptanalysis::{checks, degree, kani4, mitm3};
use eigen_seta::drbg::Drbg;
use eigen_seta::kem::EigenSeta;
use eigen_seta::params::Toy16;
use num_traits::One;

type K = EigenSeta<Toy16, 4>;

#[test]
fn key_is_a_function_of_the_secret_degree() {
    let mut rng = Drbg::for_vector("cryptanalysis-degree", 0);
    let (pk, _sk, f) = K::keygen_with_trace(&mut rng);
    let sk2 = degree::recover_key_from_degree::<Toy16, 4, _>(&pk, &f, &mut rng)
        .expect("the true degree must yield a working key");
    let (ct, k) = K::encaps(&pk, &mut rng).unwrap();
    assert_eq!(K::decaps(&sk2, &ct), k);
    let c = checks::check_degree::<Toy16, 4, _>(&f, &mut rng);
    assert!(c.all_pass(), "{c:?}");
}

#[test]
fn wrong_degrees_fail_only_at_the_end() {
    let mut rng = Drbg::for_vector("cryptanalysis-nearmiss", 0);
    let (pk, _sk, f) = K::keygen_with_trace(&mut rng);
    // Toy16 has a 50-bit degree; 32 bits of agreement leaves ~2^17 slots, of
    // which roughly 1/ln(2^174) admit Kani scalars (as in honest keygen).
    let agree = [0usize, 8, 24, 32];
    let rows = degree::near_miss_experiment::<Toy16, 4, _>(&pk, &f, &agree, 2, &mut rng);
    let truth = degree::summarise(&rows, None);
    assert!(truth.rows >= 1);
    assert!(truth.decaps_correct >= 1, "true f must decapsulate");
    assert!(
        truth.final_split >= 1,
        "true f must split at the scheduled step"
    );
    assert_eq!(truth.early_split, 0);
    for k in agree {
        let s = degree::summarise(&rows, Some(k));
        assert!(s.rows >= 1, "no samples at agreement {k}");
        assert_eq!(
            s.decaps_correct, 0,
            "a wrong degree decapsulated (agree {k})"
        );
        assert_eq!(
            s.final_split, 0,
            "a wrong degree split at the end (agree {k})"
        );
        assert_eq!(s.early_split, 0, "a wrong degree split early (agree {k})");
        // The honest schedule's first two steps are satisfied by every
        // derived key, so they carry no information about f either.
        assert_eq!(s.prodker_ok, s.rows);
        assert_eq!(s.glue_ok, s.rows);
    }
}

#[test]
fn meet_in_the_middle_recovers_the_sender_scalar_at_toy16() {
    let mut rng = Drbg::for_vector("cryptanalysis-mitm", 0);
    let (pk, _sk) = K::keygen(&mut rng);
    let (ct, _k) = K::encaps(&pk, &mut rng).unwrap();
    let r = mitm3::attack::<Toy16, 4>(&pk, &ct);
    eprintln!("{r:?}");
    assert!(r.confirmed_by_reencryption, "{r:?}");
    let s = r.recovered_s.unwrap();
    assert_eq!(K::encrypt_scalar(&pk, &s).unwrap().bytes, ct.bytes);
    // Cost shape: about 3^h + 3^(F3-h) chains in the middle phase.
    let three_h = 3u64.pow(r.h as u32);
    assert!(r.chains_from_a as u64 <= three_h + three_h / 3);
}

#[test]
fn robert_params_and_mask_oracle_at_toy16() {
    let mut rng = Drbg::for_vector("cryptanalysis-kani4", 0);
    let (e, a1, a2) = kani4::robert_params::<Toy16, 4, _>(&mut rng)
        .expect("Toy16 must admit Robert parameters 2^e = d + a1^2 + a2^2");
    let e_mask = kani4::e_mask_for(e);
    eprintln!("Robert e={e}, e_mask={e_mask}, a1={a1}, a2={a2}");
    assert!(e_mask >= e.div_ceil(2));
    assert!((&a1 % 2u32).is_one());

    let (pk, _sk) = K::keygen(&mut rng);
    let (ct, _k) = K::encaps(&pk, &mut rng).unwrap();
    // Recover s first so we can feed the honest masks (the dim-4 evaluator
    // would discover them by search; here we time one correct guess).
    let mitm = mitm3::attack::<Toy16, 4>(&pk, &ct);
    assert!(mitm.confirmed_by_reencryption, "{mitm:?}");
    let s = mitm.recovered_s.unwrap();
    let (a, b) = K::masks_for_s(&pk, &s).expect("masks");

    let r = kani4::attack::<Toy16, 4, _>(&pk, &ct, None, Some((a, b)), &mut rng);
    eprintln!("{r:?}");
    assert!(r.confirmed_by_reencryption, "{r:?}");
    assert_eq!(r.e, e);
    assert_eq!(r.e_mask, e_mask);
    assert!(r.oracle_seconds > 0.0);
    assert_eq!(
        K::encrypt_scalar(&pk, &r.recovered_s.unwrap())
            .unwrap()
            .bytes,
        ct.bytes
    );
}

#[test]
fn mask_guess_without_honest_masks_does_not_claim_recovery() {
    let mut rng = Drbg::for_vector("cryptanalysis-kani4-noguess", 0);
    let (pk, _sk) = K::keygen(&mut rng);
    let (ct, _k) = K::encaps(&pk, &mut rng).unwrap();
    let r = kani4::attack::<Toy16, 4, _>(&pk, &ct, Some(8), None, &mut rng);
    eprintln!("{r:?}");
    assert!(!r.confirmed_by_reencryption);
    assert!(r.recovered_s.is_none());
    assert!(r.guesses_tried >= 8);
    assert!(r.e > 0);
}
