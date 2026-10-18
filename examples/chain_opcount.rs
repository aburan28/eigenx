//! Exact operation counts of the theta (2,2)-chain (feature `opcount`).
//!
//! For one L1 key: the attacker's public-key degree test (one theta chain of
//! e steps, `cryptanalysis::pk_test`) and one decapsulation. Reports F_p
//! multiplications (squares included) and additions/subtractions, point
//! doublings and image evaluations, per (2,2)-step and in total.
//!
//!   cargo run --release --features opcount --example chain_opcount
use eigen_seta::cryptanalysis::pk_test::{plan_for_degree, run_pk_test_theta};
use eigen_seta::fp::opcount as fpc;
use eigen_seta::kem::EigenSeta;
use eigen_seta::params::{ParamSet, L1};
use eigen_seta::theta::opcount as thc;
use rand_core::OsRng;
use std::time::Instant;

type K = EigenSeta<L1, 21>;

fn report(label: &str, steps: u64, wall: f64) {
    let (m, a) = fpc::get();
    let (d, e, s, p) = thc::get();
    let st = steps.max(1) as f64;
    println!(
        "{label}\tsteps={steps}\tfp_mul={m}\tfp_add={a}\tdbl={d}\teval={e}\tthetasteps={s}\tprecom={p}\t\
         fp_mul_per_step={:.1}\tfp_add_per_step={:.1}\tdbl_per_step={:.2}\teval_per_step={:.2}\twall_s={wall:.4}\tus_per_step={:.1}",
        m as f64 / st,
        a as f64 / st,
        d as f64 / st,
        e as f64 / st,
        wall * 1e6 / st
    );
}

fn main() {
    let mut rng = OsRng;
    let (pk, sk, f) = K::keygen_with_jobs_trace(1);
    let plan = plan_for_degree(&f, false, L1::E, L1::E, 63, &mut rng).expect("plan");
    fpc::reset();
    thc::reset();
    let t = Instant::now();
    let ok = run_pk_test_theta::<L1, 21>(&pk, &f, &plan).unwrap_or(false);
    report("pk_test_chain", plan.e as u64, t.elapsed().as_secs_f64());
    assert!(ok);
    let (ct, k1) = K::encaps(&pk, &mut rng).unwrap();
    let prep = K::decaps_prepare(&sk, &ct).unwrap();
    fpc::reset();
    thc::reset();
    let t = Instant::now();
    let k2 = K::decaps_core(&sk, &prep, &ct);
    let wall = t.elapsed().as_secs_f64();
    let (_, _, s, _) = thc::get();
    report("decaps_core", s, wall);
    assert_eq!(k1, k2);
}
