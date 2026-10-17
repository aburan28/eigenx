//! Known-answer tests.  Keys, ciphertexts and shared secrets generated from
//! fixed DRBG seeds must match the vectors in `tests/kat/`.  Any change to
//! the algorithm, the encoding or the hash domain separation shows up here.
//!
//! Regenerate with `EIGENX_UPDATE_KAT=1 cargo test --release --test kat`
//! and review the diff.

use eigen_seta::drbg::Drbg;
use eigen_seta::kem::EigenSeta;
use eigen_seta::params::{ParamSet, L1};
use eigen_seta::pke::EigenSetaPke;
use std::fmt::Write as _;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn vectors<P: ParamSet<N>, const N: usize>(count: u64) -> String {
    let mut out = String::new();
    writeln!(out, "# Eigen-Seta known-answer vectors, set {}", P::NAME).unwrap();
    for i in 0..count {
        let mut kg = Drbg::for_vector(&format!("{}-keygen", P::NAME), i);
        let (pk, sk) = EigenSeta::<P, N>::keygen(&mut kg);
        let mut en = Drbg::for_vector(&format!("{}-encaps", P::NAME), i);
        let (ct, ss) = EigenSeta::<P, N>::encaps(&pk, &mut en).unwrap();
        assert_eq!(EigenSeta::<P, N>::decaps(&sk, &ct), ss);
        let msg = format!("EigenX KAT message {i}");
        let mut pe = Drbg::for_vector(&format!("{}-pke", P::NAME), i);
        let pct = EigenSetaPke::<P, N>::encrypt(&pk, msg.as_bytes(), b"kat", &mut pe).unwrap();
        assert_eq!(
            EigenSetaPke::<P, N>::decrypt(&sk, &pct, b"kat").as_deref(),
            Some(msg.as_bytes())
        );
        writeln!(out, "vector = {i}").unwrap();
        writeln!(out, "pk = {}", hex(&pk.bytes)).unwrap();
        writeln!(out, "sk = {}", hex(&EigenSeta::<P, N>::sk_to_bytes(&sk))).unwrap();
        writeln!(out, "ct = {}", hex(&ct.bytes)).unwrap();
        writeln!(out, "ss = {}", hex(&ss)).unwrap();
        writeln!(out, "pke_ct = {}", hex(&pct.bytes)).unwrap();
    }
    out
}

fn check<P: ParamSet<N>, const N: usize>(file: &str, count: u64) {
    let path = format!("{}/tests/kat/{file}", env!("CARGO_MANIFEST_DIR"));
    let got = vectors::<P, N>(count);
    if std::env::var_os("EIGENX_UPDATE_KAT").is_some() {
        std::fs::create_dir_all(format!("{}/tests/kat", env!("CARGO_MANIFEST_DIR"))).unwrap();
        std::fs::write(&path, &got).unwrap();
        eprintln!("updated {path}");
        return;
    }
    let want = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing KAT file {path}: {e}; run with EIGENX_UPDATE_KAT=1"));
    if want != got {
        for (ln, (w, g)) in want.lines().zip(got.lines()).enumerate() {
            if w != g {
                panic!(
                    "KAT mismatch in {file} at line {}:\n expected {w}\n got      {g}",
                    ln + 1
                );
            }
        }
        panic!("KAT mismatch in {file}: different number of lines");
    }
}

#[test]
fn kat_l1() {
    check::<L1, 21>("l1.txt", 1);
}

/// The draft NIST-style response file must decapsulate to its shared
/// secrets. This does not re-run key generation. Regenerate the file with
/// `eigenx kat` (see `docs/nist/SPECIFICATION.md`).
#[test]
fn nist_kat_l1_rsp_decaps_matches() {
    let dir = format!("{}/docs/nist/kat", env!("CARGO_MANIFEST_DIR"));
    let rsp = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("missing {dir}: {e}; run eigenx kat --set L1"))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("PQCkemKAT_L1_") && n.ends_with(".rsp"))
        })
        .unwrap_or_else(|| panic!("no PQCkemKAT_L1_*.rsp in {dir}"));
    let text = std::fs::read_to_string(&rsp).unwrap();
    assert!(
        text.contains("false submission"),
        "L1 KAT header must say a Category 1 recommendation is a false submission"
    );
    let mut pk = String::new();
    let mut sk = String::new();
    let mut ct = String::new();
    let mut ss = String::new();
    let mut saw = 0u32;
    let finish = |pk: &str, sk: &str, ct: &str, ss: &str, n: &mut u32| {
        if pk.is_empty() {
            return;
        }
        let pk_b = unhex(pk);
        let sk_b = unhex(sk);
        let ct_b = unhex(ct);
        let ss_b = unhex(ss);
        let sk = EigenSeta::<L1, 21>::sk_from_bytes(&sk_b).expect("sk");
        let got = EigenSeta::<L1, 21>::decaps(&sk, &eigen_seta::kem::Ciphertext { bytes: ct_b });
        assert_eq!(got.as_slice(), ss_b.as_slice(), "count ss");
        assert_eq!(sk.public_key().bytes, pk_b);
        *n += 1;
    };
    for line in text.lines() {
        if line.starts_with('#') || line.is_empty() {
            if line.is_empty() {
                finish(&pk, &sk, &ct, &ss, &mut saw);
                pk.clear();
                sk.clear();
                ct.clear();
                ss.clear();
            }
            continue;
        }
        if line.starts_with("count = ") || line.starts_with("seed = ") {
            continue;
        } else if let Some(v) = line.strip_prefix("pk = ") {
            pk = v.to_string();
        } else if let Some(v) = line.strip_prefix("sk = ") {
            sk = v.to_string();
        } else if let Some(v) = line.strip_prefix("ct = ") {
            ct = v.to_string();
        } else if let Some(v) = line.strip_prefix("ss = ") {
            ss = v.to_string();
        }
    }
    finish(&pk, &sk, &ct, &ss, &mut saw);
    assert!(saw >= 1, "rsp contained no vectors");
}

fn unhex(s: &str) -> Vec<u8> {
    assert!(s.len().is_multiple_of(2), "odd hex length");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
