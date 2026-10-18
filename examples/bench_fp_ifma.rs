//! F_p multiplication time per parameter set, portable vs AVX-512 IFMA path,
//! in one binary (`set_ifma`). A dependent chain of products, so it measures
//! latency; the best of nine runs of 100,000 products.
//!
//!   cargo run --release --example bench_fp_ifma
use eigen_seta::fp::{set_ifma, Fp};
use eigen_seta::params::*;
use std::hint::black_box;
use std::time::Instant;
fn t<C: eigen_seta::fp::FpConfig<N>, const N: usize>() -> f64 {
    let mut x = Fp::<C, N>::from_u64(123456789);
    let y = Fp::<C, N>::from_u64(987654321).neg();
    let it = 100_000;
    let mut best = f64::MAX;
    for _ in 0..9 {
        let s = Instant::now();
        for _ in 0..it {
            x = black_box(x * y);
        }
        best = best.min(s.elapsed().as_nanos() as f64 / it as f64);
    }
    best
}
fn main() {
    for on in [false, true] {
        set_ifma(on);
        println!(
            "ifma={on} L1={:.0} L3={:.0} L5={:.0} Q3={:.0} Q5={:.0} ns",
            t::<L1Fp, 21>(),
            t::<L3Fp, 32>(),
            t::<L5Fp, 42>(),
            t::<Q3Fp, 61>(),
            t::<Q5Fp, 82>()
        );
    }
}
