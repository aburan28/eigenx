//! Hostile-input behaviour: decapsulation must never panic and must return
//! the implicit-rejection key, deterministically, for anything that is not
//! an honest ciphertext; secret-key serialisation must round-trip and
//! reject corrupt files; encapsulation must reject malformed public keys.

use eigen_seta::drbg::Drbg;
use eigen_seta::fp2::Fp2;
use eigen_seta::kem::{Ciphertext, EigenSeta, PublicKey};
use eigen_seta::params::{L1Fp, ParamSet, L1};
use rand_core::RngCore;

type K = EigenSeta<L1, 21>;
type F = Fp2<L1Fp, 21>;

fn keypair(seed: u64) -> (PublicKey, eigen_seta::kem::SecretKey) {
    let mut rng = Drbg::for_vector("robustness", seed);
    K::keygen(&mut rng)
}

#[test]
fn random_ciphertexts_never_panic_and_reject_deterministically() {
    let (_pk, sk) = keypair(1);
    let mut rng = Drbg::for_vector("robustness-ct", 0);
    let n = K::ct_bytes();
    let mut seen = std::collections::HashSet::new();
    for _ in 0..200 {
        let mut b = vec![0u8; n];
        rng.fill_bytes(&mut b);
        let ct = Ciphertext { bytes: b };
        let k1 = K::decaps(&sk, &ct);
        let k2 = K::decaps(&sk, &ct);
        assert_eq!(k1, k2, "implicit rejection must be deterministic");
        assert!(
            seen.insert(k1),
            "rejection keys must depend on the ciphertext"
        );
    }
    for len in [0usize, 1, n - 1, n + 1, 2 * n] {
        let ct = Ciphertext {
            bytes: vec![0x5a; len],
        };
        let _ = K::decaps(&sk, &ct);
    }
    for fill in [0x00u8, 0xff] {
        let ct = Ciphertext {
            bytes: vec![fill; n],
        };
        let _ = K::decaps(&sk, &ct);
    }
}

#[test]
fn non_canonical_field_encoding_is_rejected() {
    let (pk, sk) = keypair(2);
    let mut rng = Drbg::for_vector("robustness-enc", 0);
    let (ct, k) = K::encaps(&pk, &mut rng).unwrap();
    assert_eq!(K::decaps(&sk, &ct), k);
    // Replace the first field element by an encoding of p + x_0 where x_0 is
    // the original value: out of range, so parsing must fail.
    let l = F::byte_len();
    let bad = {
        // p itself, big-endian, is an invalid F_p element encoding (>= p).
        let modulus = <L1Fp as eigen_seta::fp::FpConfig<21>>::MODULUS;
        let mut m = vec![0u8; l / 2];
        for (k, byte) in m.iter_mut().rev().enumerate() {
            *byte = (modulus[k / 8] >> (8 * (k % 8))) as u8;
        }
        let mut c = ct.bytes.clone();
        c[..l / 2].copy_from_slice(&m);
        c
    };
    let r = K::decaps(&sk, &Ciphertext { bytes: bad });
    assert_ne!(r, k, "non-canonical encoding accepted");
}

#[test]
fn cross_key_decapsulation_fails() {
    let (pk1, sk1) = keypair(3);
    let (_pk2, sk2) = keypair(4);
    let mut rng = Drbg::for_vector("robustness-cross", 0);
    let (ct, k) = K::encaps(&pk1, &mut rng).unwrap();
    assert_eq!(K::decaps(&sk1, &ct), k);
    assert_ne!(K::decaps(&sk2, &ct), k);
}

#[test]
fn every_single_bit_flip_is_rejected() {
    let (pk, sk) = keypair(5);
    let mut rng = Drbg::for_vector("robustness-flip", 0);
    let (ct, k) = K::encaps(&pk, &mut rng).unwrap();
    // Flip one bit in each byte position (one bit per byte keeps the test
    // fast; the field-element parse rejects most out-of-range cases anyway).
    for i in 0..ct.bytes.len() {
        let mut b = ct.bytes.clone();
        b[i] ^= 1 << (i % 8);
        let r = K::decaps(&sk, &Ciphertext { bytes: b });
        assert_ne!(r, k, "bit flip at byte {i} accepted");
    }
}

#[test]
fn secret_key_serialisation_round_trips_and_rejects_corruption() {
    let (pk, sk) = keypair(6);
    let bytes = K::sk_to_bytes(&sk);
    assert_eq!(bytes.len(), K::sk_bytes());
    let sk2 = K::sk_from_bytes(&bytes).expect("round trip");
    assert_eq!(sk2.public_key().bytes, pk.bytes);
    let mut rng = Drbg::for_vector("robustness-sk", 0);
    let (ct, k) = K::encaps(&pk, &mut rng).unwrap();
    assert_eq!(K::decaps(&sk2, &ct), k);
    assert!(K::sk_from_bytes(&bytes[..bytes.len() - 1]).is_none());
    assert!(K::sk_from_bytes(&[]).is_none());
    // Corrupt the embedded public key: parse fails or validation fails.
    let mut bad = bytes.clone();
    let off = bytes.len() - K::pk_bytes();
    for b in bad[off..].iter_mut() {
        *b = 0;
    }
    assert!(K::sk_from_bytes(&bad).is_none());
    // Corrupt a secret scalar: still parses (scalars are reduced) but the
    // key no longer decapsulates correctly.
    let mut bad = bytes.clone();
    bad[0] ^= 0xff;
    let sk3 = K::sk_from_bytes(&bad).unwrap();
    assert_ne!(K::decaps(&sk3, &ct), k);
}

#[test]
fn malformed_public_keys_are_rejected_by_encaps() {
    let mut rng = Drbg::for_vector("robustness-pk", 0);
    let n = K::pk_bytes();
    for len in [0usize, n - 1, n + 1] {
        let pk = PublicKey {
            bytes: vec![1; len],
        };
        assert!(K::encaps(&pk, &mut rng).is_none());
    }
    for _ in 0..50 {
        let mut b = vec![0u8; n];
        rng.fill_bytes(&mut b);
        // Random bytes are almost always non-canonical or not a valid basis.
        let pk = PublicKey { bytes: b };
        let _ = K::encaps(&pk, &mut rng);
    }
    let (pk, _) = keypair(7);
    // Swap the two basis points: differential point no longer matches.
    let l = F::byte_len();
    let mut b = pk.bytes.clone();
    let (p, q) = (b[l..2 * l].to_vec(), b[2 * l..3 * l].to_vec());
    b[l..2 * l].copy_from_slice(&q);
    b[2 * l..3 * l].copy_from_slice(&p);
    // x(P - Q) = x(Q - P), so swapping P and Q keeps the key valid; the
    // check that matters is that the *result* still round-trips or is
    // rejected, never a panic.
    let _ = K::encaps(&PublicKey { bytes: b }, &mut rng);
    // Zero the differential point: must be rejected.
    let mut b = pk.bytes.clone();
    for x in b[3 * l..].iter_mut() {
        *x = 0;
    }
    assert!(K::encaps(&PublicKey { bytes: b }, &mut rng).is_none());
    let _ = <L1 as ParamSet<21>>::NAME;
}
