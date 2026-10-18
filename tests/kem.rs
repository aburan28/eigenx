use eigen_seta::kem::{Ciphertext, EigenSeta};
use eigen_seta::params::L1;
use rand_core::OsRng;
use std::time::Instant;

fn roundtrip<P: eigen_seta::params::ParamSet<N>, const N: usize>(trials: usize) {
    let mut rng = OsRng;
    let t0 = Instant::now();
    let (pk, sk) = EigenSeta::<P, N>::keygen(&mut rng);
    eprintln!(
        "[{}] keygen {:?}; pk {} B, ct {} B; secret degree has {} bits",
        P::NAME,
        t0.elapsed(),
        pk.bytes.len(),
        EigenSeta::<P, N>::ct_bytes(),
        sk.secret_degree_bits()
    );
    for i in 0..trials {
        let t1 = Instant::now();
        let (ct, k1) = EigenSeta::<P, N>::encaps(&pk, &mut rng).unwrap();
        let te = t1.elapsed();
        let t2 = Instant::now();
        let k2 = EigenSeta::<P, N>::decaps(&sk, &ct);
        let td = t2.elapsed();
        assert_eq!(k1, k2, "decapsulation mismatch in trial {i}");
        eprintln!("[{}] trial {i}: encaps {:?}, decaps {:?}", P::NAME, te, td);
        // tampering: flip one byte of the ciphertext -> implicit rejection key
        let mut bad = ct.bytes.clone();
        let j = bad.len() - 3;
        bad[j] ^= 1;
        let k3 = EigenSeta::<P, N>::decaps(&sk, &Ciphertext { bytes: bad });
        assert_ne!(k3, k1, "tampered ciphertext accepted");
    }
}

#[test]
fn kem_l1() {
    roundtrip::<L1, 21>(1);
}

/// SplitMix64: deterministic test RNG, so two encapsulations can share coins.
struct TestRng(u64);
impl rand_core::RngCore for TestRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn fill_bytes(&mut self, d: &mut [u8]) {
        for c in d.chunks_mut(8) {
            let v = self.next_u64().to_le_bytes();
            c.copy_from_slice(&v[..c.len()]);
        }
    }
    fn try_fill_bytes(&mut self, d: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(d);
        Ok(())
    }
}
impl rand_core::CryptoRng for TestRng {}

/// `encaps_prepared` gives the same ciphertext and key as `encaps` for the
/// same coins, and repeated decapsulations through the secret key's basis
/// cache (filled by the first call, also after a clone) agree.
#[test]
fn prepared_encaps_and_cached_decaps_l1() {
    type K = EigenSeta<L1, 21>;
    let (pk, sk) = K::keygen(&mut TestRng(7));
    let ppk = K::prepare_public_key(&pk).expect("valid public key");
    for i in 0..2u64 {
        let (ct, k1) = K::encaps(&pk, &mut TestRng(100 + i)).unwrap();
        let (ct_p, k1_p) = K::encaps_prepared(&ppk, &mut TestRng(100 + i)).unwrap();
        assert_eq!(ct.bytes, ct_p.bytes, "prepared ciphertext differs");
        assert_eq!(k1, k1_p, "prepared key differs");
        assert_eq!(K::decaps(&sk, &ct), k1, "decaps {i}");
        assert_eq!(K::decaps(&sk.clone(), &ct), k1, "decaps after clone {i}");
    }
    let mut bad = pk.bytes.clone();
    bad[1] ^= 1;
    assert!(K::prepare_public_key(&eigen_seta::kem::PublicKey { bytes: bad }).is_none());
}
