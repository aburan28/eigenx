//! Test candidate secret degrees on the public key alone
//! (`cryptanalysis::pk_test`) and compare its cost with the decapsulation
//! the cost model charges per candidate.
//!
//! cargo run --release --example pk_degree_test -- L1 [wrong=40] [reps=5] [inspect=1]
//!
//! Every candidate is tested with the theta chain (accept = the chain's
//! codomain is E_A).  The first `inspect` candidates of each kind (true,
//! wrong) are also walked with the structural checks of
//! `chain::inspect_chain`, which is quadratic in e and takes minutes at L1.
//! Prints one tab-separated record per line: `keygen`, `true`, `inspect`,
//! `wrong`, `time` and `summary` rows.  Variable time; analysis only.

use eigen_seta::cryptanalysis::pk_test::{inspect_pk_test, plan_for_degree, run_pk_test_theta};
use eigen_seta::kem::EigenSeta;
use eigen_seta::ntheory::random_below;
use eigen_seta::params::{ParamSet, L1, L3, L5};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use rand_core::OsRng;
use std::time::Instant;

const MAX_U: u64 = 63;

/// The theta test, with a panic inside the chain on a wrong candidate
/// counted as a rejection.
fn theta_accepts<P: ParamSet<N>, const N: usize>(
    pk: &eigen_seta::kem::PublicKey,
    f: &BigUint,
    plan: &eigen_seta::cryptanalysis::pk_test::PkTestPlan,
) -> (bool, bool) {
    match std::panic::catch_unwind(|| run_pk_test_theta::<P, N>(pk, f, plan)) {
        Ok(Some(ok)) => (ok, false),
        _ => (false, true),
    }
}

fn run<P: ParamSet<N>, const N: usize>(wrong: usize, reps: usize, inspect: usize) {
    std::panic::set_hook(Box::new(|_| {}));
    let mut rng = OsRng;
    let t0 = Instant::now();
    let (pk, sk, f) = EigenSeta::<P, N>::keygen_with_trace(&mut rng);
    println!(
        "keygen\t{}\tE={}\tf_bits={}\tseconds={:.2}",
        P::NAME,
        P::E,
        f.bits(),
        t0.elapsed().as_secs_f64()
    );
    let e_floor = (&f * &f * 7u32).bits() as usize + 1;
    // The true degree, both assignments of the eigenvalues.
    let mut good = None;
    for flip in [false, true] {
        let Some(plan) = plan_for_degree(&f, flip, P::E, P::E, MAX_U, &mut rng) else {
            println!("true\tflip={flip}\tplan=none");
            continue;
        };
        let t = Instant::now();
        let (ok, panicked) = theta_accepts::<P, N>(&pk, &f, &plan);
        println!(
            "true\tflip={flip}\te={}\tu={}\te_floor={e_floor}\taccept={ok}\tpanic={panicked}\tseconds={:.3}",
            plan.e,
            plan.u,
            t.elapsed().as_secs_f64()
        );
        if ok && good.is_none() {
            good = Some(plan);
        }
    }
    if let Some(plan) = good.as_ref().filter(|_| inspect > 0) {
        let rep = inspect_pk_test::<P, N>(&pk, &f, plan).expect("honest pk");
        println!(
            "inspect\ttrue\te={}\tglue_ok={}\tfirst_zero_delta={:?}\trichelot={}\tfinal_split={}",
            plan.e, rep.glue_ok, rep.first_zero_delta_step, rep.richelot_steps, rep.final_split
        );
    }
    let Some(plan) = good else {
        println!("summary\ttrue degree not accepted");
        return;
    };
    // Timing: the public-key test (theta chain of length e) against one
    // full decapsulation (the per-candidate oracle of the cost model),
    // interleaved.
    let (ct, k) = EigenSeta::<P, N>::encaps(&pk, &mut rng).expect("encaps");
    for r in 0..reps {
        let t = Instant::now();
        let (ok, _) = theta_accepts::<P, N>(&pk, &f, &plan);
        let t_pk = t.elapsed().as_secs_f64();
        let t = Instant::now();
        let dk = EigenSeta::<P, N>::decaps(&sk, &ct);
        let t_dec = t.elapsed().as_secs_f64();
        println!(
            "time\trep={r}\tpk_test_e={}\tpk_test_s={t_pk:.4}\tpk_test_codomain_is_EA={ok}\tdecaps_E={}\tdecaps_s={t_dec:.4}\tdecaps_correct={}",
            plan.e,
            P::E,
            dk == k
        );
    }
    // Wrong candidates: uniform odd f' in [lo, 2 lo) prime to 21, and
    // near misses that agree with f on the low k bits.
    let (lo, _) = EigenSeta::<P, N>::secret_degree_range();
    let mut accepted = 0usize;
    let mut tested = 0usize;
    let mut extra = vec![];
    let mut cands: Vec<(String, BigUint)> = vec![];
    while cands.len() < wrong {
        let mut fp = &lo + random_below(&mut rng, &lo);
        if (&fp % 2u32).is_zero() {
            fp += 1u32;
        }
        if (&fp % 3u32).is_zero() || (&fp % 7u32).is_zero() || fp == f {
            continue;
        }
        cands.push(("uniform".into(), fp));
    }
    for kb in [16usize, 64, f.bits() as usize / 2] {
        for j in 1..=4u32 {
            let fp = &f + (BigUint::one() << kb) * j;
            if (&fp % 3u32).is_zero() || (&fp % 7u32).is_zero() {
                continue;
            }
            cands.push((format!("agree{kb}"), fp));
        }
    }
    let mut inspected = 0usize;
    for (kind, fp) in cands {
        let Some(plan) = plan_for_degree(&fp, false, P::E, P::E, MAX_U, &mut rng) else {
            println!("wrong\t{kind}\tplan=none");
            continue;
        };
        let t = Instant::now();
        let (ok, panicked) = theta_accepts::<P, N>(&pk, &fp, &plan);
        tested += 1;
        accepted += ok as usize;
        extra.push(plan.e - ((&fp * &fp * 7u32).bits() as usize + 1));
        println!(
            "wrong\t{kind}\te={}\tu={}\taccept={ok}\tpanic={panicked}\tseconds={:.3}",
            plan.e,
            plan.u,
            t.elapsed().as_secs_f64()
        );
        if inspected < inspect {
            inspected += 1;
            let rep = inspect_pk_test::<P, N>(&pk, &fp, &plan).expect("pk");
            println!(
                "inspect\twrong\t{kind}\te={}\tglue_ok={}\tfirst_zero_delta={:?}\trichelot={}\tfinal_split={}",
                plan.e, rep.glue_ok, rep.first_zero_delta_step, rep.richelot_steps, rep.final_split
            );
        }
    }
    let mean_extra = extra.iter().sum::<usize>() as f64 / extra.len().max(1) as f64;
    println!(
        "summary\t{}\ttrue_e={}\tE={}\twrong_tested={tested}\twrong_accepted={accepted}\tmean_e_above_floor={mean_extra:.2}\tmax_e_above_floor={}",
        P::NAME,
        plan.e,
        P::E,
        extra.iter().max().unwrap_or(&0)
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let set = args.get(1).map(String::as_str).unwrap_or("L1");
    let wrong = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(40);
    let reps = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(5);
    let inspect = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
    match set {
        "L1" => run::<L1, 21>(wrong, reps, inspect),
        "L3" => run::<L3, 32>(wrong, reps, inspect),
        "L5" => run::<L5, 42>(wrong, reps, inspect),
        _ => panic!("set must be L1, L3 or L5"),
    }
}
