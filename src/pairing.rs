//! Weil pairing of order n = 3^k on y^2 = x^3 + a2 x^2 + a4 x + a6 and a
//! constant-time discrete logarithm in mu_{3^k}.  The Miller loop runs over
//! the public bits of n after depressing the curve to short Weierstrass
//! (an isomorphism, so the pairing is unchanged).  Line functions are kept
//! with their exact scale factors (numerator/denominator accumulation, one
//! inversion at the end), which the Weil-pairing ratio needs because the
//! embedding degree over F_{p^2} is 1.

use crate::ct::{choice_from_bit, ConditionallySelectable};
use crate::field::Field;
use crate::scalar::Scalar;
use crate::weier::{WCurve, WPoint};
use num_bigint::BigUint;

/// f_{n,P}(Q_i) as (num, den) for each affine Q_i, for odd n with P of exact
/// order n.  The doubling chain of P is shared across every Q.  `NQ` is
/// monomorphised so the per-point loop unrolls.
pub(crate) fn miller_many<F: Field, const NQ: usize>(
    e: &WCurve<F>,
    p: &WPoint<F>,
    qs: &[WPoint<F>; NQ],
    n: &BigUint,
) -> [(F, F); NQ] {
    let (aa, _) = e.short_ab();
    let shift = e.a2 * F::from_i64(3).invert();
    let xp = p.x + shift;
    let yp = p.y;
    let mut qx = [F::zero(); NQ];
    let mut qy = [F::zero(); NQ];
    for i in 0..NQ {
        qx[i] = qs[i].x + shift;
        qy[i] = qs[i].y;
    }
    let mut x = xp;
    let mut y = yp;
    let mut z = F::one();
    let mut num = [F::one(); NQ];
    let mut den = [F::one(); NQ];
    // MSB-first bits.  A byte vector, not `Vec<bool>` (that one is bit-packed
    // and the Miller loop indexes it hundreds of times).
    let nbits = n.bits() as usize;
    let mut bitv = vec![0u8; nbits];
    for i in 0..nbits {
        bitv[nbits - 1 - i] = u8::from(n.bit(i as u64));
    }
    let last = nbits - 1;
    for idx in 1..nbits {
        let bit = bitv[idx] != 0;
        // Jacobian doubling on y^2 = x^3 + aa x + b, line evaluated at each Q.
        let xx = x.square();
        let yy = y.square();
        let yyyy = yy.square();
        let zz = z.square();
        let zz2 = zz.square();
        let s = ((x + yy).square() - xx - yyyy).double();
        let m = xx.double() + xx + aa * zz2;
        let x3 = m.square() - s.double();
        let y3 = m * (s - x3) - yyyy.double().double().double();
        let z3 = (y * z).double();
        let zzz = zz * z;
        let two_y_zzz = (y * zzz).double();
        let mzz = m * zz;
        let mx = m * x;
        let yy2 = yy.double();
        let z3s = z3.square();
        let sg = two_y_zzz;
        let sv = z3s;
        for i in 0..NQ {
            let g = two_y_zzz * qy[i] - yy2 - mzz * qx[i] + mx;
            let v = qx[i] * z3s - x3;
            let ns = num[i].square();
            let ds = den[i].square();
            num[i] = ns * g * sv;
            den[i] = ds * v * sg;
        }
        x = x3;
        y = y3;
        z = z3;
        if bit {
            if idx == last {
                // [n-1]P + P = O: vertical line through P.
                for i in 0..NQ {
                    num[i] *= qx[i] - xp;
                }
            } else {
                // Mixed addition of affine P = (xp : yp : 1).
                let zz = z.square();
                let zzz = zz * z;
                let u2 = xp * zz;
                let s2 = yp * zzz;
                let h = u2 - x;
                let hh = h.square();
                let ii = hh.double().double();
                let j = h * ii;
                let r = (s2 - y).double();
                let vv = x * ii;
                let x3 = r.square() - j - vv.double();
                let y3 = r * (vv - x3) - (y * j).double();
                let z3 = (z * h).double();
                let z3_zzz = z3 * zzz;
                let r_zzz = r * zzz;
                let r_x_z = r * x * z;
                let z3s = z3.square();
                let sg = z3_zzz;
                let sv = z3s;
                let z3y = z3 * y;
                for i in 0..NQ {
                    let g = z3_zzz * qy[i] - z3y - r_zzz * qx[i] + r_x_z;
                    let v = qx[i] * z3s - x3;
                    num[i] = num[i] * g * sv;
                    den[i] = den[i] * v * sg;
                }
                x = x3;
                y = y3;
                z = z3;
            }
        }
    }
    let mut out = [(F::zero(), F::zero()); NQ];
    for i in 0..NQ {
        out[i] = (num[i], den[i]);
    }
    out
}

/// (n1/d1) / (n2/d2), squared.  The square is a bijection on mu_n for odd n.
pub(crate) fn weil_ratio<F: Field>(fwd: (F, F), rev: (F, F)) -> F {
    let (n1, d1) = fwd;
    let (n2, d2) = rev;
    ((n1 * d2) * (d1 * n2).invert()).square()
}

/// e_n(P, Q)^2 (the square removes the (-1)^n sign and is a bijection on
/// mu_n for odd n).  Constant time in P and Q for fixed public n.
pub fn weil_sq<F: Field>(e: &WCurve<F>, p: &WPoint<F>, q: &WPoint<F>, n: &BigUint) -> F {
    let fwd = miller_many(e, p, &[*q], n)[0];
    let rev = miller_many(e, q, &[*p], n)[0];
    weil_ratio(fwd, rev)
}

/// x^(3^j).
pub fn cube_iter<F: Field>(x: &F, j: usize) -> F {
    let mut r = *x;
    for _ in 0..j {
        r = r.cube();
    }
    r
}

/// Is zeta (in mu_{3^k}) of full order 3^k?  Constant time.
pub fn full_order_3k<F: Field>(zeta: &F, k: usize) -> crate::ct::Choice {
    !cube_iter(zeta, k - 1).ct_eq(&F::one())
}

/// s with base^s = target in mu_{3^k} (base of full order).  Constant time:
/// digits come from a fixed-width Pohlig–Hellman scan (window 6), chosen by
/// constant-time comparison.  Elements of mu_{3^k} have norm 1, so
/// x^-1 = x^p = frob(x).
pub fn dlog_3k<F: Field>(base: &F, target: &F, k: usize) -> Scalar {
    // Six ternary digits per step: 3^6 = 729 comparisons, and the cubing
    // count drops from k(k-1)/2 to about k^2/12.
    const W: usize = 6;
    let mut pw = Vec::with_capacity(k);
    let mut cur = *base;
    for _ in 0..k {
        pw.push(cur);
        cur = cur.cube();
    }
    let mut acc = F::one();
    let mut digits = vec![0u64; k];
    let mut table: Vec<F> = Vec::new();
    let mut table_w = 0usize;
    let mut i = 0usize;
    while i < k {
        let w = W.min(k - i);
        if w != table_w {
            let gamma = pw[k - w];
            let mut table_len = 1usize;
            for _ in 0..w {
                table_len *= 3;
            }
            table.clear();
            table.push(F::one());
            for j in 1..table_len {
                table.push(table[j - 1] * gamma);
            }
            table_w = w;
        }
        let m = k - i - w;
        let mut h = *target * acc;
        for _ in 0..m {
            h = h.cube();
        }
        let mut digit = 0u64;
        let mut td = vec![0u64; w];
        for (j, tj) in table.iter().enumerate() {
            let eq = h.ct_eq(tj);
            digit = u64::conditional_select(&digit, &(j as u64), eq);
            // Ternary digits of the public index j; the secret one is selected.
            let mut jj = j as u64;
            for t in 0..w {
                let d = jj % 3;
                td[t] = u64::conditional_select(&td[t], &d, eq);
                jj /= 3;
            }
        }
        // acc *= base^{-digit * 3^i} = (pw[i]^digit)^{-1}.  Ten bits cover 3^6.
        let mut result = F::one();
        let mut bp = pw[i];
        for b in 0..10u32 {
            let prod = result * bp;
            result = F::conditional_select(&result, &prod, choice_from_bit((digit >> b) & 1));
            bp = bp.square();
        }
        acc *= result.frob();
        for t in 0..w {
            digits[i + t] = td[t];
        }
        i += w;
    }
    // s = sum d_i 3^i by Horner (fixed width, constant time)
    let limbs = (k * 16 / 10) / 64 + 2; // 3^k < 2^(1.585 k)
    let mut s = vec![0u64; limbs];
    for i in (0..k).rev() {
        let mut carry = digits[i] as u128;
        for w in s.iter_mut() {
            let t = (*w as u128) * 3 + carry;
            *w = t as u64;
            carry = t >> 64;
        }
    }
    Scalar { l: s }
}
