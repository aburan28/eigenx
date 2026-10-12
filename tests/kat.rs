//! Known-answer tests.  Keys, ciphertexts and shared secrets generated from
//! fixed DRBG seeds must match the vectors in `tests/kat/`.  Any change to
//! the algorithm, the encoding or the hash domain separation shows up here.
//!
//! Regenerate with `EIGENX_UPDATE_KAT=1 cargo test --release --test kat`
//! and review the diff.

use eigen_seta::drbg::Drbg;
use eigen_seta::kem::EigenSeta;
use eigen_seta::params::{ParamSet, Toy16};
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
fn kat_toy16() {
    check::<Toy16, 4>("toy16.txt", 3);
}
