//! dudect-style timing test of F_p multiplication (whichever path is active:
//! the AVX-512 IFMA kernel where available, else the portable code).
//!
//! Class 0 multiplies a fixed operand pair, class 1 a fresh random pair per
//! sample; classes are interleaved in a random order. Each sample times a
//! batch of 64 multiplications. Welch's t on raw and percentile-cropped
//! samples (Reparaz–Balasch–Verbauwhede); |t| > 4.5 flags a leak.
//!
//! Both classes generate random operands before each batch; class 0 then
//! overwrites them with the fixed pair, so the work done outside the timed
//! region is the same. `portable` as a third argument disables IFMA.
//!
//!   cargo run --release --example dudect_fp -- <L1|L3|L5> [samples_per_class] [portable]
use eigen_seta::fp::{ifma_active, Fp, FpConfig};
use eigen_seta::params::{L1Fp, L3Fp, L5Fp};
use rand_core::{OsRng, RngCore};
use std::hint::black_box;
use std::time::Instant;

fn random<C: FpConfig<N>, const N: usize>() -> Fp<C, N> {
    loop {
        let mut b = vec![0u8; C::BYTES];
        OsRng.fill_bytes(&mut b);
        let (x, ok) = Fp::<C, N>::from_bytes_be(&b);
        if bool::from(ok) {
            return x;
        }
    }
}

fn welch(a: &[f64], b: &[f64]) -> f64 {
    let m = |x: &[f64]| x.iter().sum::<f64>() / x.len() as f64;
    let v = |x: &[f64], mu: f64| {
        x.iter().map(|y| (y - mu) * (y - mu)).sum::<f64>() / (x.len() as f64 - 1.0)
    };
    let (ma, mb) = (m(a), m(b));
    (ma - mb) / (v(a, ma) / a.len() as f64 + v(b, mb) / b.len() as f64).sqrt()
}

fn run<C: FpConfig<N>, const N: usize>(n: usize) {
    const BATCH: usize = 64;
    // a fixed pair drawn once at random (structured values such as 1 and -1
    // give a large |t| even on the instruction-count-checked portable code)
    let fixed = (random::<C, N>(), random::<C, N>());
    let mut s = [Vec::with_capacity(n), Vec::with_capacity(n)];
    while s[0].len() < n || s[1].len() < n {
        let class = (OsRng.next_u32() & 1) as usize;
        if s[class].len() >= n {
            continue;
        }
        // Identical preparation for both classes: draw random operands, then
        // one pass over the buffer that keeps them (class 1) or replaces them
        // with the fixed pair (class 0). Only the values differ.
        let mut ops: Vec<(Fp<C, N>, Fp<C, N>)> = (0..BATCH)
            .map(|_| (random::<C, N>(), random::<C, N>()))
            .collect();
        for op in ops.iter_mut() {
            let r = *op;
            *op = if class == 0 { fixed } else { r };
        }
        let t = Instant::now();
        for (a, b) in &ops {
            black_box(black_box(*a) * black_box(*b));
        }
        s[class].push(t.elapsed().as_nanos() as f64);
    }
    let mut all: Vec<f64> = s[0].iter().chain(s[1].iter()).copied().collect();
    all.sort_by(|a, b| a.partial_cmp(b).unwrap());
    print!(
        "{} ifma={} n={n} t_raw={:+.2}",
        C::NAME,
        ifma_active(),
        welch(&s[0], &s[1])
    );
    for pct in [0.5, 0.75, 0.9, 0.99] {
        let cut = all[(pct * (all.len() - 1) as f64) as usize];
        let a: Vec<f64> = s[0].iter().copied().filter(|x| *x <= cut).collect();
        let b: Vec<f64> = s[1].iter().copied().filter(|x| *x <= cut).collect();
        print!(" t_p{:.0}={:+.2}", pct * 100.0, welch(&a, &b));
    }
    println!();
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let n = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(200_000);
    if a.get(3).map(String::as_str) == Some("portable") {
        eigen_seta::fp::set_ifma(false);
    }
    match a.get(1).map(String::as_str).unwrap_or("L1") {
        "L1" => run::<L1Fp, 21>(n),
        "L3" => run::<L3Fp, 32>(n),
        "L5" => run::<L5Fp, 42>(n),
        _ => panic!("L1 | L3 | L5"),
    }
}
