//! Mumford [`decaps_chain`] vs theta [`decaps_chain_theta`] on L1.
//!
//! Builds the same eigen-kernel sample as `tests/chain.rs` and checks that
//! both paths return isomorphic codomains (equal j-invariants, matching
//! the domain) and payload x-coordinates compatible up to factor swap,
//! Weierstrass isomorphism and sign.

use eigen_seta::chain::{decaps_chain, decaps_chain_theta};
use eigen_seta::cm::{
    chain3_vartime, eigenvector_2k, mul_vartime, point_of_order, torsion_basis, Omega,
};
use eigen_seta::ct::ConstantTimeEq;
use eigen_seta::field::Field;
use eigen_seta::ntheory::{modpos, sqrt_mod_2power, two_squares, v2};
use eigen_seta::params::{ParamSet, L1};
use eigen_seta::weier::{complementary_2_torsion_xs, Isog2, ToMontgomery, WCurve, WIso, WPoint};
use num_bigint::{BigInt, BigUint};
use rand_core::{OsRng, RngCore};

mod common;
use common::*;

/// True when affine x-coordinates match (infinity matches infinity).
fn x_eq<Fld: eigen_seta::field::Field>(a: &WPoint<Fld>, b: &WPoint<Fld>) -> bool {
    let both_inf = bool::from(a.inf & b.inf);
    let both_fin = bool::from(!a.inf & !b.inf);
    both_inf || (both_fin && bool::from(a.x.ct_eq(&b.x)))
}

/// Compare payload pairs up to (factor swap) × (codomain iso) × (component swap).
fn payloads_match<Fld: eigen_seta::field::Field>(
    e_ref: &WCurve<Fld>,
    e_oth: &WCurve<Fld>,
    ref_imgs: &[(WPoint<Fld>, WPoint<Fld>)],
    oth_imgs: &[(WPoint<Fld>, WPoint<Fld>)],
) -> bool {
    assert_eq!(ref_imgs.len(), oth_imgs.len());
    let sig = WIso::find(e_oth, e_ref);
    for swap in [false, true] {
        for s in [sig, sig.negate()] {
            let mut ok = true;
            for (r, o) in ref_imgs.iter().zip(oth_imgs.iter()) {
                let (o0, o1) = if swap { (o.1, o.0) } else { (o.0, o.1) };
                let m0 = s.eval(&o0);
                let m1 = s.eval(&o1);
                if !(x_eq(&r.0, &m0) && x_eq(&r.1, &m1) || x_eq(&r.0, &m1) && x_eq(&r.1, &m0)) {
                    ok = false;
                    break;
                }
            }
            if ok {
                return true;
            }
        }
    }
    false
}

#[test]
fn montgomery_normalise_preserves_j() {
    let a = F::from_counter(7);
    let ec = WCurve::from_montgomery(a);
    let xs = complementary_2_torsion_xs(&ec, F::zero());
    // Kernel at x=0 on Montgomery; complementary are the other two.
    let phi = Isog2::new(&ec, &F::zero());
    let img = phi.eval(&WPoint::new(xs[0], F::zero()));
    let img2 = phi.eval(&WPoint::new(xs[1], F::zero()));
    let r = if bool::from(!img.inf) { img.x } else { img2.x };
    let tm = ToMontgomery::from_torsion(&phi.codomain, r);
    assert_eq!(
        tm.curve.j_invariant(),
        phi.codomain.j_invariant(),
        "ToMontgomery changed the j-invariant"
    );
    assert!(bool::from(tm.curve.a6.ct_eq(&F::zero())));
    assert!(bool::from(tm.curve.a4.ct_eq(&F::one())));
}

#[test]
fn theta_chain_matches_mumford_l1() {
    let mut rng = OsRng;
    let ed = ed_curve();
    let om = Omega::find(&ed, &mut rng);
    let big_e = L1::E;
    let t = om.trace;
    let mod_e = BigUint::from(1u32) << big_e;
    let s_d = sqrt_mod_2power(
        &((BigUint::from(1u32) << (big_e + 2)) - 7u32),
        big_e as u64 + 2,
    );
    let s_d = BigInt::from(s_d);
    let mu = modpos(&((BigInt::from(t) + &s_d) / 2), &mod_e);
    let nu = modpos(&((BigInt::from(t) - &s_d) / 2), &mod_e);
    let w = |x: &WPoint<F>| om.eval(x);
    let p = eigenvector_2k(&ed, &w, &nu, big_e, &mut rng);
    let q = eigenvector_2k(&ed, &w, &mu, big_e, &mut rng);
    assert!(
        bool::from(om.eval(&p).ct_eq(&mul_vartime(&ed, &p, &mu))),
        "P not an eigenvector"
    );
    assert!(
        bool::from(om.eval(&q).ct_eq(&mul_vartime(&ed, &q, &nu))),
        "Q not an eigenvector"
    );

    let kdeg = 5usize;
    let d = BigUint::from(3u32).pow(kdeg as u32);
    let k3 = point_of_order(&ed, 3, kdeg, &mut rng);
    let (y1, y2) = torsion_basis(&ed, 3, L1::F3, &mut rng);
    let mut pts = [p, q, y1, y2];
    let ec = chain3_vartime(&ed, &k3, kdeg, &mut pts);
    let [pc, qc, x1, x2] = pts;
    let sd_mod = modpos(&(BigInt::from(d.clone()) * &s_d), &mod_e);

    let mut found = None;
    'search: for ep in (40..=big_e).rev() {
        let m7 = &d * &d * 7u32;
        if (BigUint::from(1u32) << ep) <= m7 {
            continue;
        }
        let m = (BigUint::from(1u32) << ep) - m7;
        if let Some((x, y)) = two_squares(&m, &mut rng, 1000) {
            let (c, cp) = if &x % 2u32 == BigUint::from(0u32) {
                (x, y)
            } else {
                (y, x)
            };
            let n2 = BigInt::from(1u8) << ep;
            for sc in [1i32, -1] {
                for scp in [1i32, -1] {
                    let c = BigInt::from(c.clone()) * sc;
                    let cp = BigInt::from(cp.clone()) * scp;
                    let lam_s = (BigInt::from(sd_mod.clone()) + &c) % &n2;
                    if v2(&(&cp - &lam_s)).max(v2(&(&cp + &lam_s))) == 2 {
                        found = Some((ep, c, cp));
                        break 'search;
                    }
                }
            }
        }
    }
    let (ep, c, cp) = found.expect("no admissible chain length");
    let n2 = BigUint::from(1u32) << ep;
    let lam_s = modpos(&(BigInt::from(sd_mod.clone()) + &c), &n2);
    let lam_t = modpos(&(&c - BigInt::from(sd_mod.clone())), &n2);
    let cpm = modpos(&cp, &n2);
    let shift = BigUint::from(1u32) << (big_e - ep);
    let s = mul_vartime(&ec, &pc, &shift);
    let tt = mul_vartime(&ec, &qc, &shift);
    let k1 = (
        mul_vartime(&ec, &s, &cpm),
        mul_vartime(&ec, &s, &lam_s).neg(),
    );
    let k2 = (
        mul_vartime(&ec, &tt, &cpm),
        mul_vartime(&ec, &tt, &lam_t).neg(),
    );
    let o = WPoint::infinity();
    let payload = [(x1, o), (o, x1), (x2, o)];

    let t0 = std::time::Instant::now();
    let (fc_m, imgs_m) = decaps_chain(&ec, &k1, &k2, ep, &payload);
    eprintln!("mumford chain length {ep}: {:?}", t0.elapsed());
    assert_eq!(
        fc_m.j_invariant(),
        ec.j_invariant(),
        "Mumford codomain is not E_C"
    );

    let t1 = std::time::Instant::now();
    let (fc_t, imgs_t) = decaps_chain_theta(&ec, &k1, &k2, ep, &payload);
    eprintln!("theta   chain length {ep}: {:?}", t1.elapsed());

    assert_eq!(
        fc_t.j_invariant(),
        ec.j_invariant(),
        "theta codomain is not E_C"
    );
    assert!(
        payloads_match(&fc_m, &fc_t, &imgs_m, &imgs_t),
        "payload images disagree up to swap/iso/sign"
    );
    let _ = rng.next_u32();
}
