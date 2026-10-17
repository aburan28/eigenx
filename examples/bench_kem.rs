//! Benchmark of the whole KEM and PKE on one parameter set, for the
//! performance white paper.
//!
//! cargo run --release --example bench_kem -- <L1|L3|L5|Q3|Q5> [keys=3] [trials=5] [jobs=1]
//!
//! For each of `keys` fresh keys (`keygen_with_jobs_trace(jobs)`, wall and
//! process CPU time), it runs `trials` encapsulations and decapsulations,
//! one authenticated PKE round trip of a 1 KiB message, and one public-key
//! degree test (`cryptanalysis::pk_test`, the attacker's per-candidate
//! oracle) whose chain time is divided by its length.  It also times F_p and
//! F_p² multiplication and F_p inversion in the same binary, so the chain
//! cost can be stated in F_p² multiplications per (2,2)-step: the unit the
//! security cost model charges (`security::chain_gates_log2`, 2^8 per step).
//!
//! One tab-separated record per line: `sizes`, `field`, `keygen`, `op`,
//! `pke`, `chain`.  Variable time where noted; analysis only.

use eigen_seta::cryptanalysis::pk_test::{plan_for_degree, run_pk_test_theta};
use eigen_seta::fp::Fp;
use eigen_seta::fp2::Fp2;
use eigen_seta::kem::EigenSeta;
use eigen_seta::params::*;
use eigen_seta::pke::EigenSetaPke;
use rand_core::OsRng;
use std::hint::black_box;
use std::time::Instant;

/// User + system CPU seconds of this process (Linux `/proc/self/stat`).
fn cpu_seconds() -> f64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    // Fields after the command name, which is parenthesised and may hold spaces.
    let rest = s.rsplit_once(')').map(|(_, r)| r).unwrap_or("");
    let f: Vec<&str> = rest.split_whitespace().collect();
    let ticks = |i: usize| f.get(i).and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0);
    // utime and stime are fields 14 and 15 of the full line, 12 and 13 here.
    (ticks(11) + ticks(12)) / 100.0
}

fn field<P: ParamSet<N>, const N: usize>() -> (f64, f64, f64) {
    type F<P, const N: usize> = Fp<<P as ParamSet<N>>::Cfg, N>;
    type F2<P, const N: usize> = Fp2<<P as ParamSet<N>>::Cfg, N>;
    let it = 20000;
    let mut x = F::<P, N>::from_u64(123456789);
    let mut fp_mul = f64::MAX;
    for _ in 0..15 {
        let t = Instant::now();
        for _ in 0..it {
            x = black_box(x * x + F::<P, N>::one());
        }
        fp_mul = fp_mul.min(t.elapsed().as_nanos() as f64 / it as f64);
    }
    let mut y = F2::<P, N>::new(F::<P, N>::from_u64(987654321), F::<P, N>::from_u64(12345));
    let c = F2::<P, N>::new(F::<P, N>::from_u64(3), F::<P, N>::from_u64(5));
    let mut fp2_mul = f64::MAX;
    for _ in 0..15 {
        let t = Instant::now();
        for _ in 0..it {
            y = black_box(y * c + c);
        }
        fp2_mul = fp2_mul.min(t.elapsed().as_nanos() as f64 / it as f64);
    }
    let inv_it = 200;
    let t = Instant::now();
    for _ in 0..inv_it {
        x = black_box(x.invert() + F::<P, N>::one());
    }
    let fp_inv = t.elapsed().as_nanos() as f64 / inv_it as f64;
    (fp_mul, fp2_mul, fp_inv)
}

fn go<P: ParamSet<N>, const N: usize>(keys: usize, trials: usize, jobs: usize) {
    let mut rng = OsRng;
    println!(
        "sizes\t{}\tpk={}\tct={}\tsk={}\tss=32\tfile_header=8\tpke_overhead={}",
        P::NAME,
        EigenSeta::<P, N>::pk_bytes(),
        EigenSeta::<P, N>::ct_bytes(),
        EigenSeta::<P, N>::sk_bytes(),
        EigenSeta::<P, N>::ct_bytes() + 12 + 16
    );
    let (fp_mul, fp2_mul, fp_inv) = field::<P, N>();
    println!(
        "field\t{}\tfp_mul_ns={fp_mul:.1}\tfp2_mul_ns={fp2_mul:.1}\tfp_inv_ns={fp_inv:.0}",
        P::NAME
    );
    let msg = vec![0x5au8; 1024];
    for k in 0..keys {
        let (w0, c0) = (Instant::now(), cpu_seconds());
        let (pk, sk, f) = EigenSeta::<P, N>::keygen_with_jobs_trace(jobs);
        println!(
            "keygen\t{}\tkey={k}\tjobs={jobs}\twall_s={:.3}\tcpu_s={:.2}\tf_bits={}\tsk_serialised={}",
            P::NAME,
            w0.elapsed().as_secs_f64(),
            cpu_seconds() - c0,
            f.bits(),
            EigenSeta::<P, N>::sk_to_bytes(&sk).len()
        );
        for t in 0..trials {
            let w = Instant::now();
            let (ct, k1) = EigenSeta::<P, N>::encaps(&pk, &mut rng).expect("encaps");
            let te = w.elapsed().as_secs_f64();
            let w = Instant::now();
            let k2 = EigenSeta::<P, N>::decaps(&sk, &ct);
            let td = w.elapsed().as_secs_f64();
            println!(
                "op\t{}\tkey={k}\ttrial={t}\tencaps_s={te:.4}\tdecaps_s={td:.4}\tok={}",
                P::NAME,
                k1 == k2
            );
        }
        let w = Instant::now();
        let ct = EigenSetaPke::<P, N>::encrypt(&pk, &msg, b"bench", &mut rng).expect("pke encrypt");
        let tenc = w.elapsed().as_secs_f64();
        let w = Instant::now();
        let pt = EigenSetaPke::<P, N>::decrypt(&sk, &ct, b"bench");
        let tdec = w.elapsed().as_secs_f64();
        println!(
            "pke\t{}\tkey={k}\tmsg=1024\tct={}\tencrypt_s={tenc:.4}\tdecrypt_s={tdec:.4}\tok={}",
            P::NAME,
            ct.bytes.len(),
            pt.as_deref() == Some(&msg[..])
        );
        if let Some(plan) = plan_for_degree(&f, false, P::E, P::E, 63, &mut rng) {
            let w = Instant::now();
            let ok = run_pk_test_theta::<P, N>(&pk, &f, &plan).unwrap_or(false);
            let tc = w.elapsed().as_secs_f64();
            let per_step = tc / plan.e as f64;
            println!(
                "chain\t{}\tkey={k}\te={}\twall_s={tc:.4}\tper_step_us={:.1}\tfp2_mul_per_step={:.0}\taccept={ok}",
                P::NAME,
                plan.e,
                per_step * 1e6,
                per_step * 1e9 / fp2_mul
            );
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let set = args.get(1).map(String::as_str).unwrap_or("L1");
    let keys = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
    let trials = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(5);
    let jobs = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
    match set {
        "L1" => go::<L1, 21>(keys, trials, jobs),
        "L3" => go::<L3, 32>(keys, trials, jobs),
        "L5" => go::<L5, 42>(keys, trials, jobs),
        "Q3" => go::<Q3, 61>(keys, trials, jobs),
        "Q5" => go::<Q5, 82>(keys, trials, jobs),
        _ => panic!("set must be L1, L3, L5, Q3 or Q5"),
    }
}
