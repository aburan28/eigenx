//! The AVX-512 IFMA multiplier (`eigen_fp_ifma`) must return exactly the
//! portable product for every parameter set: random operands, the extremes
//! 0, 1, p - 1, and a long chain of products. Skips with a note when this
//! CPU has no IFMA, since then both sides would be the portable code.
use eigen_seta::fp::{ifma_active, ifma_supports, set_ifma, set_ifma64, Fp, FpConfig};
use eigen_seta::fp2::Fp2;
use eigen_seta::params::{L1Fp, L3Fp, L5Fp, Q3Fp, Q5Fp};
use rand_core::{OsRng, RngCore};
use std::sync::Mutex;

// set_ifma is process-wide; tests in this binary must not interleave.
static LOCK: Mutex<()> = Mutex::new(());

fn random<C: FpConfig<N>, const N: usize>() -> Fp<C, N> {
    let mut b = vec![0u8; C::BYTES];
    OsRng.fill_bytes(&mut b);
    b[0] &= 0x7f >> (8 * C::BYTES - C::BITS).min(7);
    let (x, ok) = Fp::<C, N>::from_bytes_be(&b);
    if bool::from(ok) {
        x
    } else {
        random::<C, N>()
    }
}

fn check<C: FpConfig<N>, const N: usize>() {
    assert!(
        ifma_supports::<C, N>(),
        "{}: IFMA path not supported",
        C::NAME
    );
    let mut xs: Vec<Fp<C, N>> = vec![
        Fp::from_u64(0),
        Fp::one(),
        Fp::one().neg(),
        Fp::from_u64(2).neg(),
    ];
    for _ in 0..3000 {
        xs.push(random::<C, N>());
    }
    let pairs: Vec<(Fp<C, N>, Fp<C, N>)> = xs
        .iter()
        .zip(xs.iter().rev())
        .map(|(a, b)| (*a, *b))
        .chain(xs.iter().map(|a| (*a, *a)))
        .collect();
    set_ifma(false);
    let want: Vec<Vec<u8>> = pairs.iter().map(|(a, b)| (*a * *b).to_bytes_be()).collect();
    let mut c = xs[7];
    for _ in 0..5000 {
        c = c * xs[9] + Fp::one();
    }
    let want_chain = c.to_bytes_be();
    set_ifma(true);
    let got: Vec<Vec<u8>> = pairs.iter().map(|(a, b)| (*a * *b).to_bytes_be()).collect();
    let mut c = xs[7];
    for _ in 0..5000 {
        c = c * xs[9] + Fp::one();
    }
    assert_eq!(want, got, "{}: IFMA product differs", C::NAME);
    // F_p^2 products and squares go through Fp::mul3 and Fp::mul2.
    let f2: Vec<Fp2<C, N>> = xs.chunks(2).map(|c| Fp2::new(c[0], c[1])).collect();
    set_ifma(false);
    let w2: Vec<Vec<u8>> = f2
        .windows(2)
        .flat_map(|w| [(w[0] * w[1]).to_bytes_be(), w[0].square().to_bytes_be()])
        .collect();
    set_ifma(true);
    let g2: Vec<Vec<u8>> = f2
        .windows(2)
        .flat_map(|w| [(w[0] * w[1]).to_bytes_be(), w[0].square().to_bytes_be()])
        .collect();
    assert_eq!(w2, g2, "{}: IFMA F_p^2 product or square differs", C::NAME);
    assert_eq!(
        want_chain,
        c.to_bytes_be(),
        "{}: IFMA chain differs",
        C::NAME
    );
}

#[test]
fn ifma_matches_portable_all_sets() {
    let _g = LOCK.lock().unwrap();
    if !ifma_active() {
        eprintln!("no AVX-512 IFMA on this CPU: nothing to compare");
        return;
    }
    // both operand conversions: in-register (VBMI) and scalar
    for vbmi in [true, false] {
        set_ifma64(vbmi);
        check::<L1Fp, 21>();
        check::<L3Fp, 32>();
        check::<L5Fp, 42>();
        check::<Q3Fp, 61>();
        check::<Q5Fp, 82>();
    }
    set_ifma64(true);
    set_ifma(true);
}
