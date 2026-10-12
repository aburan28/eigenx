//! Run the toy-scale cryptanalysis experiments and print a report.
//!
//!   cargo run --release --example cryptanalysis -- [keys] [trials]
//!
//! `keys` key pairs are generated (default 3); the near-miss experiment
//! uses `trials` wrong degrees per agreement level (default 4).  All
//! randomness comes from a fixed DRBG seed so the report is reproducible.
use eigen_seta::cryptanalysis::{checks, degree, mitm3};
use eigen_seta::drbg::Drbg;
use eigen_seta::kem::EigenSeta;
use eigen_seta::params::{ParamSet, Toy16};
use std::time::Instant;

type K = EigenSeta<Toy16, 4>;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let keys: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let trials: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
    let e = <Toy16 as ParamSet<4>>::E;
    let agree = [0usize, 4, 8, 16, 24, 32, 40, 48];
    println!(
        "Eigen-Seta toy-scale cryptanalysis report, set {} (E = {e}, F3 = {})",
        <Toy16 as ParamSet<4>>::NAME,
        <Toy16 as ParamSet<4>>::F3
    );
    println!("keys {keys}, near-miss trials per level {trials}\n");
    let mut rng = Drbg::for_vector("cryptanalysis-report", 0);
    let mut all_rows = vec![];
    let mut checks_pass = 0;
    let mut degree_recoveries = 0;
    let mut mitm_ok = 0;
    let mut mitm_secs = 0.0;
    let mut mitm_chains = 0u64;
    for i in 0..keys {
        let t0 = Instant::now();
        let (pk, _sk, f) = K::keygen_with_trace(&mut rng);
        println!(
            "key {i}: keygen {:.2}s, f has {} bits",
            t0.elapsed().as_secs_f64(),
            f.bits()
        );
        let c = checks::check_degree::<Toy16, 4, _>(&f, &mut rng);
        println!("  structural checks: {c:?}");
        checks_pass += c.all_pass() as usize;
        let t1 = Instant::now();
        let rec = degree::recover_key_from_degree::<Toy16, 4, _>(&pk, &f, &mut rng);
        println!(
            "  key from true degree: {} ({:.2}s per guess incl. one decapsulation)",
            if rec.is_some() { "recovered" } else { "FAILED" },
            t1.elapsed().as_secs_f64()
        );
        degree_recoveries += rec.is_some() as usize;
        let t2 = Instant::now();
        let rows = degree::near_miss_experiment::<Toy16, 4, _>(&pk, &f, &agree, trials, &mut rng);
        println!(
            "  near-miss experiment: {} rows in {:.1}s",
            rows.len(),
            t2.elapsed().as_secs_f64()
        );
        all_rows.extend(rows);
        let (ct, _k) = K::encaps(&pk, &mut rng).unwrap();
        let r = mitm3::attack::<Toy16, 4>(&pk, &ct);
        println!(
            "  MITM on 3^{}: h = {}, chains {} + {} + {}, {:.1}s, recovered {}",
            <Toy16 as ParamSet<4>>::F3,
            r.h,
            r.chains_from_a,
            r.chains_from_c,
            r.chains_extend,
            r.seconds,
            r.confirmed_by_reencryption
        );
        mitm_ok += r.confirmed_by_reencryption as usize;
        mitm_secs += r.seconds;
        mitm_chains += r.chains_from_a + r.chains_from_c + r.chains_extend;
    }
    println!("\nNear-miss summary (all keys): agree_bits rows decaps_ok prodker_ok glue_ok early_split final_split mean_richelot_steps");
    let t = degree::summarise(&all_rows, None);
    println!(
        "  true-f    {:>4} {:>9} {:>10} {:>7} {:>11} {:>11} {:>8.1}",
        t.rows,
        t.decaps_correct,
        t.prodker_ok,
        t.glue_ok,
        t.early_split,
        t.final_split,
        t.mean_richelot_steps
    );
    for k in agree {
        let s = degree::summarise(&all_rows, Some(k));
        println!(
            "  k = {:<5} {:>4} {:>9} {:>10} {:>7} {:>11} {:>11} {:>8.1}",
            k,
            s.rows,
            s.decaps_correct,
            s.prodker_ok,
            s.glue_ok,
            s.early_split,
            s.final_split,
            s.mean_richelot_steps
        );
    }
    println!("\nTotals: structural checks {checks_pass}/{keys}, key-from-degree {degree_recoveries}/{keys}, MITM {mitm_ok}/{keys} ({:.1}s, {} chains total; per chain {:.1} us)",
        mitm_secs, mitm_chains, 1e6 * mitm_secs / mitm_chains.max(1) as f64);
}
