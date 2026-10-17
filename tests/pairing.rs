mod common;
use common::*;
use eigen_seta::cm::{mul_vartime, torsion_basis};
use eigen_seta::pairing::{dlog_3k, full_order_3k, weil_sq};
use eigen_seta::params::{ParamSet, L1};
use eigen_seta::weier::{WCurve, WPoint};
use num_bigint::BigUint;
use rand_core::{OsRng, RngCore};

#[test]
fn weil_bilinear_and_dlog() {
    let mut rng = OsRng;
    let ed = ed_curve();
    let k = L1::F3;
    let n = BigUint::from(3u32).pow(k as u32);
    let (p, q) = torsion_basis(&ed, 3, k, &mut rng);
    let e = weil_sq(&ed, &p, &q, &n);
    assert!(
        bool::from(full_order_3k(&e, k)),
        "pairing of a basis not of full order"
    );
    assert_eq!(cube_n(&e, k), F::one(), "pairing not in mu_n");
    for _ in 0..5 {
        let a = BigUint::from(rng.next_u64()) % &n;
        let b = BigUint::from(rng.next_u64()) % &n;
        let ap = mul_vartime(&ed, &p, &a);
        let bq = mul_vartime(&ed, &q, &b);
        let lhs = weil_sq(&ed, &ap, &bq, &n);
        let rhs = e.pow_public(&(&a * &b % &n).to_u64_digits());
        assert_eq!(lhs, rhs, "bilinearity");
        let s = dlog_3k(&e, &lhs, k);
        assert_eq!(s.to_biguint() % &n, &a * &b % &n, "dlog");
        // alternating
        let pq = ed.add(&ap, &bq);
        let e1 = weil_sq(&ed, &pq, &q, &n);
        let e2 = weil_sq(&ed, &ap, &q, &n);
        assert_eq!(e1, e2, "e(aP + bQ, Q) = e(aP, Q)");
    }
}

fn cube_n(x: &F, k: usize) -> F {
    eigen_seta::pairing::cube_iter(x, k)
}

/// Translating x by t is an isomorphism, so the Weil pairing is unchanged.
/// The Miller loop depresses the curve by a2/3; this is the check that the
/// shift agrees with the a2 = 0 model.
#[test]
fn weil_translation_invariant() {
    let mut rng = OsRng;
    let ed = ed_curve();
    let k = L1::F3;
    let n = BigUint::from(3u32).pow(k as u32);
    let (p, q) = torsion_basis(&ed, 3, k, &mut rng);
    let t = F::from_i64(5);
    let a2 = t.mul_small(3).neg();
    let a4 = t.square().mul_small(3).add(&ed.a4);
    let a6 = t.square().mul(&t).neg().sub(&ed.a4.mul(&t)).add(&ed.a6);
    let et = WCurve::new(a2, a4, a6);
    let map = |pt: WPoint<F>| WPoint::new(pt.x.add(&t), pt.y);
    let e0 = weil_sq(&ed, &p, &q, &n);
    let e1 = weil_sq(&et, &map(p), &map(q), &n);
    assert_eq!(e0, e1, "translation changed the Weil pairing");
}
