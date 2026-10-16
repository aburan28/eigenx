#![allow(dead_code)]
#[allow(unused_imports)]
use eigen_seta::field::Field;
use eigen_seta::fp::Fp;
use eigen_seta::fp2::Fp2;
use eigen_seta::params::{L1, ParamSet};
use eigen_seta::weier::WCurve;
use num_bigint::BigUint;

pub type Cfg = <L1 as ParamSet<21>>::Cfg;
pub type F = Fp2<Cfg, 21>;

pub fn fp_hex(h: &str) -> Fp<Cfg, 21> {
    let n = BigUint::parse_bytes(h.as_bytes(), 16).unwrap();
    let mut l = [0u64; 21];
    for (i, d) in n.to_u64_digits().iter().enumerate() {
        l[i] = *d;
    }
    Fp::from_canonical_limbs(l)
}

pub fn ed_curve() -> WCurve<F> {
    WCurve::new(
        F::zero(),
        Fp2::from_fp(fp_hex(L1::ED_A4_HEX)),
        Fp2::from_fp(fp_hex(L1::ED_A6_HEX)),
    )
}
