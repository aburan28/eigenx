//! Meet-in-the-middle recovery of the sender's isogeny.
//!
//! The ciphertext reveals the codomain E_C of the 3^F3-isogeny φ: E_A → E_C
//! with kernel ⟨P_d + s Q_d⟩.  The generic attack splits φ into a 3^h part
//! from E_A and a 3^(F3-h) part from E_C and matches j-invariants in the
//! middle: cost about 3^h + 3^(F3-h) isogeny computations and 3^h stored
//! j-invariants, minimised at h = F3/2, i.e. d^(1/2).  Once s mod 3^h is
//! known, the remaining digits follow from the matched middle curve; here
//! they are simply enumerated (3^(F3-h) more chains, same order of cost) and
//! the result is confirmed by re-encrypting s, which the attacker can do
//! because the masks are derived from s and the public key.
//!
//! This module exists to measure the attack, not to speed it up. At L1
//! (F3 = 162) it needs 3^81 ≈ 2^128 steps.

use crate::cm::mul_vartime;
use crate::field::Field;
use crate::kem::{Ciphertext, EigenSeta, PublicKey};
use crate::mont::{ladder3pt, MontCurve, ProjCurve, XPoint};
use crate::params::ParamSet;
use crate::scalar::Scalar;
use crate::weier::{WCurve, WPoint};
use num_bigint::BigUint;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct MitmResult {
    pub recovered_s: Option<BigUint>,
    pub confirmed_by_reencryption: bool,
    /// Number of 3^h-isogeny chains computed from E_A.
    pub chains_from_a: u64,
    /// Number of 3^(F3-h)-isogeny chains computed from E_C.
    pub chains_from_c: u64,
    /// Number of full-length chains used to extend s mod 3^h to s.
    pub chains_extend: u64,
    pub seconds: f64,
    pub h: usize,
}

type Fq<P, const N: usize> = crate::fp2::Fp2<<P as ParamSet<N>>::Cfg, N>;

/// All cyclic subgroups of order 3^h of ⟨P, Q⟩ (P, Q of order 3^h): the
/// kernels P + sQ for s in [0, 3^h) and Q + 3tP for t in [0, 3^(h-1)).
/// Returns x-coordinates of generators together with a label (s or
/// 3^h + t).
fn enumerate_kernels<Fl: Field>(
    w: &WCurve<Fl>,
    p: &WPoint<Fl>,
    q: &WPoint<Fl>,
    h: usize,
    pc: &ProjCurve<Fl>,
) -> Vec<(XPoint<Fl>, BigUint)> {
    let three_h = BigUint::from(3u32).pow(h as u32);
    let bits = three_h.bits() as usize;
    let limbs = bits.div_ceil(64) + 1;
    let pq = w.sub(p, q);
    let (xp, xq, xpq) = (
        XPoint::from_affine(p.x),
        XPoint::from_affine(q.x),
        XPoint::from_affine(pq.x),
    );
    let mut out = Vec::with_capacity(bits);
    let mut s = BigUint::default();
    while s < three_h {
        let sc = Scalar::from_biguint(&s, limbs);
        out.push((ladder3pt(&xp, &xq, &xpq, &sc, bits, pc), s.clone()));
        s += 1u32;
    }
    let three_h1 = &three_h / 3u32;
    let mut t = BigUint::default();
    while t < three_h1 {
        let sc = Scalar::from_biguint(&(&t * 3u32), limbs);
        out.push((ladder3pt(&xq, &xp, &xpq, &sc, bits, pc), &three_h + &t));
        t += 1u32;
    }
    out
}

fn j_key<Fl: Field>(c: &ProjCurve<Fl>) -> Vec<u8> {
    MontCurve::new(c.affine_a()).j_invariant().to_bytes()
}

/// Run the attack against `ct` under `pk`.
pub fn attack<P: ParamSet<N>, const N: usize>(pk: &PublicKey, ct: &Ciphertext) -> MitmResult {
    let t0 = Instant::now();
    let f3 = P::F3;
    let h = f3 / 2;
    let hc = f3 - h;
    let mut res = MitmResult {
        recovered_s: None,
        confirmed_by_reencryption: false,
        chains_from_a: 0,
        chains_from_c: 0,
        chains_extend: 0,
        seconds: 0.0,
        h,
    };
    let parse = |b: &[u8]| -> Option<[Fq<P, N>; 4]> {
        let l = <Fq<P, N> as Field>::byte_len();
        if b.len() != 4 * l {
            return None;
        }
        let mut out = [<Fq<P, N> as Field>::zero(); 4];
        for i in 0..4 {
            let (v, ok) = <Fq<P, N> as Field>::from_bytes(&b[i * l..(i + 1) * l]);
            if !bool::from(ok) {
                return None;
            }
            out[i] = v;
        }
        Some(out)
    };
    let (Some(pkf), Some(ctf)) = (parse(&pk.bytes), parse(&ct.bytes)) else {
        res.seconds = t0.elapsed().as_secs_f64();
        return res;
    };
    let a_a = pkf[0];
    let a_c = ctf[0];
    let Some((pd, qd)) = EigenSeta::<P, N>::public_basis_3(&a_a) else {
        return res;
    };
    let Some((pc3, qc3)) = EigenSeta::<P, N>::public_basis_3(&a_c) else {
        return res;
    };
    let wa = WCurve::from_montgomery(a_a);
    let wc = WCurve::from_montgomery(a_c);
    let pa = ProjCurve::from_affine(a_a);
    let pcc = ProjCurve::from_affine(a_c);
    // Stage 1: from E_A, all 3^h-isogenies.
    let cof_a = BigUint::from(3u32).pow(hc as u32);
    let (pa_h, qa_h) = (mul_vartime(&wa, &pd, &cof_a), mul_vartime(&wa, &qd, &cof_a));
    let mut table: HashMap<Vec<u8>, BigUint> = HashMap::new();
    for (xk, label) in enumerate_kernels(&wa, &pa_h, &qa_h, h, &pa) {
        let cod = crate::isog3::chain3(&pa, &xk, h, &mut []);
        res.chains_from_a += 1;
        table.insert(j_key(&cod), label);
    }
    // Stage 2: from E_C, all 3^(F3-h)-isogenies, look for a collision.
    let cof_c = BigUint::from(3u32).pow(h as u32);
    let (pc_h, qc_h) = (
        mul_vartime(&wc, &pc3, &cof_c),
        mul_vartime(&wc, &qc3, &cof_c),
    );
    let mut hit: Option<BigUint> = None;
    for (xk, _label) in enumerate_kernels(&wc, &pc_h, &qc_h, hc, &pcc) {
        let cod = crate::isog3::chain3(&pcc, &xk, hc, &mut []);
        res.chains_from_c += 1;
        if let Some(l) = table.get(&j_key(&cod)) {
            hit = Some(l.clone());
            break;
        }
    }
    let Some(label) = hit else {
        res.seconds = t0.elapsed().as_secs_f64();
        return res;
    };
    // The honest kernel is P_d + s Q_d, whose 3^h-part is the kernel
    // P_h + (s mod 3^h) Q_h with P_h = [3^(F3-h)] P_d: so the label must be
    // of the first family (label < 3^h) and equals s mod 3^h.
    let three_h = BigUint::from(3u32).pow(h as u32);
    if label >= three_h {
        res.seconds = t0.elapsed().as_secs_f64();
        return res;
    }
    // Stage 3: extend s mod 3^h to s by enumerating the high digits and
    // comparing codomains; confirm by re-encryption.
    let d = BigUint::from(3u32).pow(f3 as u32);
    let bits = d.bits() as usize;
    let limbs = bits.div_ceil(64) + 1;
    let pqd = wa.sub(&pd, &qd);
    let (xp, xq, xpq) = (
        XPoint::from_affine(pd.x),
        XPoint::from_affine(qd.x),
        XPoint::from_affine(pqd.x),
    );
    let target_j = j_key(&pcc);
    let mut s = label.clone();
    while s < d {
        let sc = Scalar::from_biguint(&s, limbs);
        let xk = ladder3pt(&xp, &xq, &xpq, &sc, bits, &pa);
        let cod = crate::isog3::chain3(&pa, &xk, f3, &mut []);
        res.chains_extend += 1;
        if j_key(&cod) == target_j {
            if let Some(ct2) = EigenSeta::<P, N>::encrypt_scalar(pk, &s) {
                if ct2.bytes == ct.bytes {
                    res.recovered_s = Some(s.clone());
                    res.confirmed_by_reencryption = true;
                    break;
                }
            }
        }
        s += &three_h;
    }
    res.seconds = t0.elapsed().as_secs_f64();
    res
}
