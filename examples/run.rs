//! cargo run --release --example run -- <L1|L3|L5|Q3|Q5> [trials]
use eigen_seta::kem::EigenSeta;
use eigen_seta::params::*;
use rand_core::OsRng;
use std::time::Instant;

fn go<P: ParamSet<N>, const N: usize>(trials: usize) {
    let mut rng = OsRng;
    let t0 = Instant::now();
    let (pk, sk) = EigenSeta::<P, N>::keygen(&mut rng);
    println!(
        "{} keygen_s {:.3} pk_bytes {} ct_bytes {} f_bits {}",
        P::NAME,
        t0.elapsed().as_secs_f64(),
        pk.bytes.len(),
        EigenSeta::<P, N>::ct_bytes(),
        sk.secret_degree_bits()
    );
    let mut ok = 0;
    for i in 0..trials {
        let t1 = Instant::now();
        let (ct, k1) = EigenSeta::<P, N>::encaps(&pk, &mut rng).unwrap();
        let te = t1.elapsed().as_secs_f64();
        let t2 = Instant::now();
        let k2 = EigenSeta::<P, N>::decaps(&sk, &ct);
        let td = t2.elapsed().as_secs_f64();
        ok += (k1 == k2) as usize;
        println!(
            "{} trial {} encaps_s {:.4} decaps_s {:.3} ok {}",
            P::NAME,
            i,
            te,
            td,
            k1 == k2
        );
    }
    println!(
        "{} RESULT {}/{} decapsulations correct",
        P::NAME,
        ok,
        trials
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let set = args.get(1).map(|s| s.as_str()).unwrap_or("L1");
    let trials = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
    match set {
        "L1" => go::<L1, 21>(trials),
        "L3" => go::<L3, 32>(trials),
        "L5" => go::<L5, 42>(trials),
        "Q3" => go::<Q3, 61>(trials),
        "Q5" => go::<Q5, 82>(trials),
        _ => panic!("unknown set"),
    }
}
