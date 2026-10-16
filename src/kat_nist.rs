//! NIST-style known-answer files for the KEM.
//!
//! The DRBG is the SHAKE256 generator in [`crate::drbg`], not NIST's
//! AES-256-CTR `randombytes`. Seeds are 48 bytes, drawn in order from a
//! master DRBG whose seed is `EigenSeta-NIST-KAT-v1-{set}`. Each vector then
//! runs `keygen` and `encaps` on one DRBG initialised with that seed.
//!
//! `keygen` draws an extra encapsulation seed for its correctness self-test.
//! Those bytes are part of `keygen_coins` in the intermediate file. A match
//! requires the same self-test draw.
//!
//! This module does not assign a NIST category. The claim line written into
//! every file is the one in [`claim_line`].

use crate::drbg::Drbg;
use crate::kem::EigenSeta;
use crate::params::ParamSet;
use num_bigint::BigUint;
use rand_core::{CryptoRng, RngCore};
use std::fs;
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

pub const SEED_BYTES: usize = 48;

/// Files written for one parameter set. Names follow
/// `PQCkemKAT_{set}_{sk_bytes}.{req,int,rsp}`.
#[derive(Clone, Debug)]
pub struct WrittenKat {
    pub req: PathBuf,
    pub intermediate: PathBuf,
    pub rsp: PathBuf,
    pub count: u64,
    pub sk_bytes: usize,
    pub encaps_seed_bytes: usize,
}

/// Sentence recorded in the KAT header for this set.
///
/// L1 and L3 must not be described as NIST categories 1 and 3.
pub fn claim_line(set: &str) -> &'static str {
    match set {
        "L1" => {
            "L1 meets a NIST gate-count style Tier A argument only. It is not NIST Category 1 under a 2^λ query bar. The binding quantum attack is Grover on mask-guess, about 2^{λ/2} queries. A submission that recommends L1 as NIST Category 1 is a false submission."
        }
        "L3" => {
            "L3 meets a NIST gate-count style Tier A argument only. It is not NIST Category 3 under a 2^λ query bar. The binding quantum attack is Grover on mask-guess, about 2^{λ/2} queries. A submission that recommends L3 as NIST Category 3 is a false submission."
        }
        "L5" => {
            "L5 is claimed at ≥ 2^128 quantum queries and ≥ 2^256 classical operations: quantum category 1 and classical category 5. The binding quantum attack is Grover on mask-guess; the cost model reports about 2^{128.5} queries. L5 is not a quantum category 5 set."
        }
        "Q3" => {
            "Q3 is claimed as category 3: ≥ 2^192 quantum queries against every known attack in the cost model. QROM IND-CCA is not claimed."
        }
        "Q5" => {
            "Q5 is claimed as category 5: ≥ 2^256 quantum queries against every known attack in the cost model. QROM IND-CCA is not claimed."
        }
        _ => "See docs/nist/SPECIFICATION.md. This draft does not add a category for an unnamed set.",
    }
}

/// Encapsulation seed length: `ceil(d_bits/8) + ceil(λ/8)`, with `d = 3^F3`
/// and `d_bits = d.bits()` as in `EigenSeta::encaps`.
pub fn encaps_seed_len<P: ParamSet<N>, const N: usize>() -> usize {
    let d = BigUint::from(3u32).pow(P::F3 as u32);
    (d.bits() as usize).div_ceil(8) + P::LAMBDA.div_ceil(8)
}

/// The `count` request seeds, in order, for this set.
pub fn entropy_seeds<P: ParamSet<N>, const N: usize>(count: u64) -> Vec<[u8; SEED_BYTES]> {
    let mut master = Drbg::new(format!("EigenSeta-NIST-KAT-v1-{}", P::NAME).as_bytes());
    let mut seeds = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let mut seed = [0u8; SEED_BYTES];
        master.fill_bytes(&mut seed);
        seeds.push(seed);
    }
    seeds
}

/// Generate `count` KAT vectors and write `.req`, `.int` and `.rsp`.
///
/// `on_vector` is called with the vector index before that vector's key
/// generation. The function does not print timings.
pub fn write_kem_kat<P: ParamSet<N>, const N: usize>(
    dir: &Path,
    count: u64,
    mut on_vector: impl FnMut(u64),
) -> Result<WrittenKat, String> {
    if count == 0 {
        return Err("KAT count must be at least 1".into());
    }
    fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let sk_len = EigenSeta::<P, N>::sk_bytes();
    let pk_len = EigenSeta::<P, N>::pk_bytes();
    let ct_len = EigenSeta::<P, N>::ct_bytes();
    let enc_len = encaps_seed_len::<P, N>();
    let stem = format!("PQCkemKAT_{}_{sk_len}", P::NAME);
    let req_path = dir.join(format!("{stem}.req"));
    let int_path = dir.join(format!("{stem}.int"));
    let rsp_path = dir.join(format!("{stem}.rsp"));

    let header = header::<P, N>(count, sk_len, pk_len, ct_len, enc_len);
    let mut req = header.clone();
    let mut intermediate = header.clone();
    let mut rsp = header;

    let seeds = entropy_seeds::<P, N>(count);
    for (i, seed) in seeds.iter().enumerate() {
        let i = i as u64;
        on_vector(i);
        let mut rng = TapRng {
            inner: Drbg::new(seed),
            log: Vec::new(),
        };
        let (pk, sk) = EigenSeta::<P, N>::keygen(&mut rng);
        let keygen_coins = rng.take();
        let (ct, mut ss) = EigenSeta::<P, N>::encaps(&pk, &mut rng)
            .ok_or_else(|| format!("vector {i}: encapsulation rejected the public key"))?;
        let encaps_coins = rng.take();
        if encaps_coins.len() != enc_len {
            return Err(format!(
                "vector {i}: encapsulation drew {} bytes, expected {enc_len} (ceil(d_bits/8)+ceil(λ/8))",
                encaps_coins.len()
            ));
        }
        let ss_dec = EigenSeta::<P, N>::decaps(&sk, &ct);
        if ss_dec != ss {
            return Err(format!(
                "vector {i}: decapsulation did not return the encapsulated shared secret"
            ));
        }
        let mut sk_bytes = EigenSeta::<P, N>::sk_to_bytes(&sk);
        if sk_bytes.len() != sk_len || pk.bytes.len() != pk_len || ct.bytes.len() != ct_len {
            sk_bytes.zeroize();
            return Err(format!(
                "vector {i}: object length does not match the parameter set"
            ));
        }
        let seed_hex = hex_upper(seed);
        let pk_hex = hex_upper(&pk.bytes);
        let sk_hex = hex_upper(&sk_bytes);
        let ct_hex = hex_upper(&ct.bytes);
        let ss_hex = hex_upper(&ss);
        sk_bytes.zeroize();
        ss.zeroize();

        req.push_str(&blank_record(i, &seed_hex));
        intermediate.push_str(&format!(
            "count = {i}\nseed = {seed_hex}\nkeygen_coins = {}\nencaps_coins = {}\n\n",
            hex_upper(&keygen_coins),
            hex_upper(&encaps_coins)
        ));
        rsp.push_str(&format!(
            "count = {i}\nseed = {seed_hex}\npk = {pk_hex}\nsk = {sk_hex}\nct = {ct_hex}\nss = {ss_hex}\n\n"
        ));
    }

    fs::write(&req_path, req).map_err(|e| format!("cannot write {}: {e}", req_path.display()))?;
    fs::write(&int_path, intermediate)
        .map_err(|e| format!("cannot write {}: {e}", int_path.display()))?;
    fs::write(&rsp_path, rsp).map_err(|e| format!("cannot write {}: {e}", rsp_path.display()))?;
    Ok(WrittenKat {
        req: req_path,
        intermediate: int_path,
        rsp: rsp_path,
        count,
        sk_bytes: sk_len,
        encaps_seed_bytes: enc_len,
    })
}

fn header<P: ParamSet<N>, const N: usize>(
    count: u64,
    sk_len: usize,
    pk_len: usize,
    ct_len: usize,
    enc_len: usize,
) -> String {
    format!(
        "\
# Eigen-Seta KEM known-answer file (draft, not an upload)
# set = {name}
# pk_bytes = {pk_len}
# sk_bytes = {sk_len}
# ct_bytes = {ct_len}
# ss_bytes = {ss}
# encaps_seed_bytes = {enc_len}
# count = {count}
# drbg = SHAKE256 domain EigenSeta-DRBG (src/drbg.rs), not NIST AES-256-CTR
# master_seed = EigenSeta-NIST-KAT-v1-{name}
# seed_bytes = {seed_len}
# keygen_coins includes the keygen self-test encapsulation draw
# {claim}
# QROM IND-CCA is not claimed. See docs/nist/COVER.md.
#
",
        name = P::NAME,
        ss = crate::kem::SS_BYTES,
        seed_len = SEED_BYTES,
        claim = claim_line(P::NAME),
    )
}

fn blank_record(i: u64, seed_hex: &str) -> String {
    format!("count = {i}\nseed = {seed_hex}\npk =\nsk =\nct =\nss =\n\n")
}

fn hex_upper(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &x in bytes {
        s.push(HEX[(x >> 4) as usize] as char);
        s.push(HEX[(x & 0xf) as usize] as char);
    }
    s
}

struct TapRng<R> {
    inner: R,
    log: Vec<u8>,
}

impl<R> TapRng<R> {
    fn take(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.log)
    }
}

impl<R: RngCore> RngCore for TapRng<R> {
    fn next_u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        self.fill_bytes(&mut b);
        u32::from_le_bytes(b)
    }
    fn next_u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        self.fill_bytes(&mut b);
        u64::from_le_bytes(b)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.inner.fill_bytes(dest);
        self.log.extend_from_slice(dest);
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl<R: CryptoRng + RngCore> CryptoRng for TapRng<R> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::L1;

    #[test]
    fn l1_claim_is_not_category_1() {
        let s = claim_line("L1");
        assert!(s.contains("false submission"));
        assert!(s.contains("not NIST Category 1"));
        assert!(s.contains("Tier A"));
        assert!(!claim_line("L5").contains("false submission"));
        assert!(claim_line("L5").contains("quantum category 1"));
        assert!(claim_line("L5").contains("classical category 5"));
    }

    #[test]
    fn l1_entropy_seeds_are_stable_and_not_a_toy() {
        assert_eq!(L1::NAME, "L1");
        assert_eq!(L1::F3, 162);
        assert_eq!(encaps_seed_len::<L1, 21>(), 49);
        let a = entropy_seeds::<L1, 21>(2);
        let b = entropy_seeds::<L1, 21>(2);
        assert_eq!(a, b);
        assert_ne!(a[0], a[1]);
    }
}
