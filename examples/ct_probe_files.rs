//! Constant-time probe on fixed inputs, for comparing two builds on the same
//! secret keys and ciphertexts.
//!
//!   ct_probe_files gen <dir>        write sk<k>.bin, ct<k>_<valid|foreign>.bin, k = 1, 2
//!   ct_probe_files <run|prep> <dir> <k> <valid|foreign>
//!
//! `run` parses the key and ciphertext, runs `decaps_prepare` (public,
//! variable time) and then the secret phase in `probe_core`; `prep` stops
//! after `decaps_prepare`.  Under `valgrind --tool=callgrind`, the total
//! instruction count of `run` minus that of `prep` on the same inputs is
//! the secret phase's count over all threads (`--toggle-collect` counts
//! only the thread that enters `probe_core`, and per-thread files cannot
//! separate the phases because thread ids are reused).  Equal counts across
//! keys and across valid vs FO-rejected ciphertexts show control flow
//! independent of secrets.
use eigen_seta::kem::{Ciphertext, DecapsPrep, EigenSeta, SecretKey};
use eigen_seta::params::{L1Fp, L1};
use rand_core::{CryptoRng, RngCore};

/// SplitMix64: deterministic test RNG (NOT for production keys).
struct TestRng(u64);
impl RngCore for TestRng {
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
impl CryptoRng for TestRng {}

type K = EigenSeta<L1, 21>;

#[inline(never)]
fn probe_core(
    sk: &SecretKey,
    prep: &DecapsPrep<eigen_seta::fp2::Fp2<L1Fp, 21>>,
    ct: &Ciphertext,
) -> [u8; 32] {
    K::decaps_core(
        std::hint::black_box(sk),
        std::hint::black_box(prep),
        std::hint::black_box(ct),
    )
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = std::path::Path::new(&a[2]);
    match a[1].as_str() {
        "gen" => {
            for k in 1..=2u64 {
                let (pk, sk) = K::keygen(&mut TestRng(1000 + k));
                let (pk2, _) = K::keygen(&mut TestRng(9000 + k));
                let (ct, ss) = K::encaps(&pk, &mut TestRng(2000 + k)).unwrap();
                let (ctf, _) = K::encaps(&pk2, &mut TestRng(3000 + k)).unwrap();
                std::fs::write(dir.join(format!("sk{k}.bin")), K::sk_to_bytes(&sk)).unwrap();
                std::fs::write(dir.join(format!("ct{k}_valid.bin")), &ct.bytes).unwrap();
                std::fs::write(dir.join(format!("ss{k}_valid.bin")), ss).unwrap();
                std::fs::write(dir.join(format!("ct{k}_foreign.bin")), &ctf.bytes).unwrap();
            }
        }
        "run" | "prep" => {
            let (k, mode) = (&a[3], &a[4]);
            let sk = K::sk_from_bytes(&std::fs::read(dir.join(format!("sk{k}.bin"))).unwrap())
                .expect("secret key");
            let ct = Ciphertext {
                bytes: std::fs::read(dir.join(format!("ct{k}_{mode}.bin"))).unwrap(),
            };
            let prep = K::decaps_prepare(&sk, &ct).expect("well-formed ciphertext");
            if a[1] == "prep" {
                println!("key {k} {mode} prepared");
                return;
            }
            let ss = probe_core(&sk, &prep, &ct);
            let want = std::fs::read(dir.join(format!("ss{k}_valid.bin"))).unwrap();
            println!(
                "key {k} {mode} accepted {} ss {}",
                ss[..] == want[..],
                ss.iter().map(|b| format!("{b:02x}")).collect::<String>()
            );
        }
        _ => panic!("gen | run | prep"),
    }
}
