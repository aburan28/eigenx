//! AVX-512 IFMA Montgomery multiplication in radix 2^52, for primes with
//! p ≡ −1 (mod 2^52).
//!
//! This crate is the only place in eigenx that uses `unsafe`, and only to
//! call `#[target_feature]` intrinsics and to load and store vectors. The
//! public functions are safe. They run the vector code only after detecting
//! AVX-512F and AVX-512IFMA at run time, and otherwise return `false`.
//!
//! Constant time: every loop has a fixed trip count and no branch or address
//! depends on the operands. The final reduction is a masked subtraction.
//!
//! Representation. Numbers are `nl` limbs of 52 bits in a slice of `8 * W`
//! u64 words, where W is the number of 512-bit registers and `nl ≤ 8 W`.
//! Words at or above `nl` must be zero, and so must bits 52–63 of every limb.
//! `mont_mul` returns a · b · 2^(−52 nl) mod p, reduced below p. It requires
//! `a < 2^(52 nl)` and `b < p`.

#![deny(unsafe_op_in_unsafe_fn)]

const MASK: u64 = (1 << 52) - 1;

/// Whether this CPU can run [`mont_mul`]. A public property of the machine;
/// `std` caches the query.
pub fn available() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        std::arch::is_x86_feature_detected!("avx512f")
            && std::arch::is_x86_feature_detected!("avx512ifma")
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// `out = a · b · 2^(−52 nl) mod p` with W 512-bit registers per operand.
///
/// - `p` holds the modulus and `p1` holds p + 1, both in radix 2^52.
/// - `z1` is the index of the first register in which p + 1 has a non-zero
///   limb.
/// - All slices have length `8 * W`, and `nl ≤ 8 * W`.
///
/// Returns `false`, leaving `out` untouched, when the CPU lacks IFMA.
///
/// # Panics
/// If a slice has the wrong length or `nl > 8 * W` or `z1 > W`.
pub fn mont_mul<const W: usize>(
    out: &mut [u64],
    a: &[u64],
    b: &[u64],
    p: &[u64],
    p1: &[u64],
    nl: usize,
    z1: usize,
) -> bool {
    let len = 8 * W;
    assert!(out.len() == len && a.len() == len && b.len() == len);
    assert!(W <= 16, "at most 16 registers");
    assert!(p.len() == len && p1.len() == len && nl <= len && z1 <= W);
    if !available() {
        return false;
    }
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: AVX-512F and AVX-512IFMA were detected just above, and every
        // slice has length 8 * W, so the eight-word loads and stores at
        // offsets 8k, k < W, stay in bounds.
        unsafe { x86::mont_mul::<W>(out, a, b, p, p1, nl, z1) };
        true
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (out, a, b, p, p1, nl, z1);
        false
    }
}

/// K independent products `outs[j] = as_[j] · bs[j] · 2^(−52 nl) mod p` in
/// one interleaved loop. The K dependency chains overlap, which hides the
/// latency of extracting each Montgomery digit. Same contract as
/// [`mont_mul`] for every operand.
///
/// # Panics
/// As [`mont_mul`].
pub fn mont_mul_k<const W: usize, const K: usize>(
    outs: [&mut [u64]; K],
    as_: [&[u64]; K],
    bs: [&[u64]; K],
    p: &[u64],
    p1: &[u64],
    nl: usize,
    z1: usize,
) -> bool {
    let len = 8 * W;
    assert!(W <= 16, "at most 16 registers");
    assert!(p.len() == len && p1.len() == len && nl <= len && z1 <= W);
    for j in 0..K {
        assert!(outs[j].len() == len && as_[j].len() == len && bs[j].len() == len);
    }
    if !available() {
        return false;
    }
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: AVX-512F and AVX-512IFMA were detected above, and every
        // slice holds 8 * W words.
        unsafe { x86::mont_mul_k::<W, K>(outs, as_, bs, p, p1, nl, z1) };
        true
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (outs, as_, bs, p, p1, nl, z1);
        false
    }
}

/// Tables that let the kernel read 64-bit limbs directly into radix-2^52
/// registers (AVX-512 VBMI). Built once per field by [`ConvTables::new`];
/// every entry depends only on the field size and shift, never on data.
///
/// Register k loads up to 64 bytes from byte offset `base[k]` of the input.
/// Byte j of lane q is input byte `perm[k][8q + j]`, relative to that base,
/// or zero where bit 8q + j of `keep[k]` is clear. Lane q then shifts right
/// by `ctrl[k][8q]` bits (multishift) and is masked to 52 bits.
#[derive(Clone, Copy, Debug)]
pub struct ConvTables {
    pub base: [usize; 16],
    pub perm: [[u8; 64]; 16],
    pub keep: [u64; 16],
    pub ctrl: [[u8; 64]; 16],
}

impl ConvTables {
    /// Tables for an input of `n64` 64-bit limbs read as `nl` 52-bit limbs,
    /// with the value shifted up `shift` bits (`shift < 52`).
    pub const fn new(n64: usize, nl: usize, shift: usize) -> Self {
        let mut t = ConvTables { base: [0; 16], perm: [[0; 64]; 16], keep: [0; 16], ctrl: [[0; 64]; 16] };
        let regs = nl.div_ceil(8);
        let mut k = 0;
        while k < regs && k < 16 {
            // first byte any lane of this register needs (may be negative)
            let s0 = (52 * 8 * k) as isize - shift as isize;
            let b0 = s0.div_euclid(8);
            let base = if b0 < 0 { 0 } else { b0 as usize };
            t.base[k] = base;
            let mut q = 0;
            while q < 8 {
                let i = 8 * k + q;
                if i < nl {
                    let si = (52 * i) as isize - shift as isize;
                    let bi = si.div_euclid(8);
                    let ri = si.rem_euclid(8) as u8;
                    let mut j = 0;
                    while j < 8 {
                        let src = bi + j as isize - base as isize;
                        let byte = bi + j as isize;
                        if src >= 0 && src < 64 && byte >= 0 && (byte as usize) < 8 * n64 {
                            t.perm[k][8 * q + j] = src as u8;
                            t.keep[k] |= 1 << (8 * q + j);
                        }
                        t.ctrl[k][8 * q + j] = ri + 8 * j as u8;
                        j += 1;
                    }
                }
                q += 1;
            }
            k += 1;
        }
        t
    }
}

/// [`mont_mul_k`] reading the operands as 64-bit limbs through `ta`
/// (left operands) and `tb` (right operands). Results are radix-2^52 limbs
/// in `outs`, as for [`mont_mul_k`]. Needs AVX-512 VBMI as well; returns
/// `false` without it.
///
/// # Panics
/// As [`mont_mul_k`], and if an input is longer than `8 * W` words.
pub fn mont_mul64_k<const W: usize, const K: usize>(
    outs: [&mut [u64]; K],
    as_: [&[u64]; K],
    bs: [&[u64]; K],
    ta: &ConvTables,
    tb: &ConvTables,
    p: &[u64],
    p1: &[u64],
    nl: usize,
    z1: usize,
) -> bool {
    let len = 8 * W;
    assert!(W <= 16, "at most 16 registers");
    assert!(p.len() == len && p1.len() == len && nl <= len && z1 <= W);
    for j in 0..K {
        assert!(outs[j].len() == len && as_[j].len() <= len && bs[j].len() <= len);
        assert!(as_[j].len() == bs[j].len());
    }
    if !available64() {
        return false;
    }
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: AVX-512F, IFMA and VBMI were detected above. Loads of the
        // 64-bit inputs are byte-masked to their length (masked loads do not
        // touch bytes outside the mask), and outputs hold 8 * W words.
        unsafe { x86::mont_mul64_k::<W, K>(outs, as_, bs, ta, tb, p, p1, nl, z1) };
        true
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (outs, as_, bs, ta, tb, p, p1, nl, z1);
        false
    }
}

/// Whether this CPU can run [`mont_mul64_k`] (IFMA plus VBMI).
pub fn available64() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        available()
            && std::arch::is_x86_feature_detected!("avx512vbmi")
            && std::arch::is_x86_feature_detected!("avx512bw")
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use super::{ConvTables, MASK};
    use core::arch::x86_64::*;

    /// Operand-scanning Montgomery multiplication (Gueron–Krasnov style).
    ///
    /// Each iteration i adds a · b_i to the accumulator. It then takes
    /// m = acc_0 mod 2^52, which is the Montgomery digit because p ≡ −1
    /// (mod 2^52). Since acc + m p = acc − m + m (p + 1), subtracting m
    /// clears limb 0 exactly, and m (p + 1) only touches registers z1 and
    /// above. The accumulator then shifts down one limb. High halves of the
    /// products belong one limb up, so they are added after the shift.
    ///
    /// # Safety
    /// The CPU must support AVX-512F and AVX-512IFMA, and every slice must
    /// have length at least 8 * W.
    #[target_feature(enable = "avx512f,avx512ifma")]
    pub(super) unsafe fn mont_mul<const W: usize>(
        out: &mut [u64],
        a: &[u64],
        b: &[u64],
        p: &[u64],
        p1: &[u64],
        nl: usize,
        z1: usize,
    ) {
        let z = _mm512_setzero_si512();
        let mut av = [z; W];
        let mut pv = [z; W];
        let mut acc = [z; W];
        for k in 0..W {
            // SAFETY: k < W and the slices hold 8 * W words (caller contract).
            unsafe {
                av[k] = _mm512_loadu_si512(a.as_ptr().add(8 * k).cast());
                pv[k] = _mm512_loadu_si512(p1.as_ptr().add(8 * k).cast());
            }
        }
        for &bi in b.iter().take(nl) {
            let bv = _mm512_set1_epi64(bi as i64);
            for k in 0..W {
                acc[k] = _mm512_madd52lo_epu64(acc[k], av[k], bv);
            }
            let a0 = _mm_cvtsi128_si64(_mm512_castsi512_si128(acc[0])) as u64;
            let mv = _mm512_set1_epi64((a0 & MASK) as i64);
            for k in z1..W {
                acc[k] = _mm512_madd52lo_epu64(acc[k], pv[k], mv);
            }
            let carry = _mm512_maskz_set1_epi64(1, (a0 >> 52) as i64);
            for k in 0..W {
                let next = if k + 1 < W { acc[k + 1] } else { z };
                acc[k] = _mm512_alignr_epi64::<1>(next, acc[k]);
            }
            acc[0] = _mm512_add_epi64(acc[0], carry);
            for k in 0..W {
                acc[k] = _mm512_madd52hi_epu64(acc[k], av[k], bv);
            }
            for k in z1..W {
                acc[k] = _mm512_madd52hi_epu64(acc[k], pv[k], mv);
            }
        }
        for k in 0..W {
            // SAFETY: as for the loads.
            unsafe { _mm512_storeu_si512(out.as_mut_ptr().add(8 * k).cast(), acc[k]) };
        }
        finish::<W>(out, p);
    }

    /// [`mont_mul`] for K independent products, interleaved step by step.
    ///
    /// # Safety
    /// As [`mont_mul`]: AVX-512F and AVX-512IFMA, and slices of 8 * W words.
    #[target_feature(enable = "avx512f,avx512ifma")]
    pub(super) unsafe fn mont_mul_k<const W: usize, const K: usize>(
        outs: [&mut [u64]; K],
        as_: [&[u64]; K],
        bs: [&[u64]; K],
        p: &[u64],
        p1: &[u64],
        nl: usize,
        z1: usize,
    ) {
        let z = _mm512_setzero_si512();
        let mut av = [[z; W]; K];
        let mut pv = [z; W];
        let mut acc = [[z; W]; K];
        for k in 0..W {
            // SAFETY: k < W and every slice holds 8 * W words.
            unsafe {
                pv[k] = _mm512_loadu_si512(p1.as_ptr().add(8 * k).cast());
                for j in 0..K {
                    av[j][k] = _mm512_loadu_si512(as_[j].as_ptr().add(8 * k).cast());
                }
            }
        }
        for i in 0..nl {
            let mut bv = [z; K];
            let mut mv = [z; K];
            let mut cv = [z; K];
            for j in 0..K {
                bv[j] = _mm512_set1_epi64(bs[j][i] as i64);
                for k in 0..W {
                    acc[j][k] = _mm512_madd52lo_epu64(acc[j][k], av[j][k], bv[j]);
                }
            }
            for j in 0..K {
                let a0 = _mm_cvtsi128_si64(_mm512_castsi512_si128(acc[j][0])) as u64;
                mv[j] = _mm512_set1_epi64((a0 & MASK) as i64);
                cv[j] = _mm512_maskz_set1_epi64(1, (a0 >> 52) as i64);
            }
            for j in 0..K {
                for k in z1..W {
                    acc[j][k] = _mm512_madd52lo_epu64(acc[j][k], pv[k], mv[j]);
                }
                for k in 0..W {
                    let next = if k + 1 < W { acc[j][k + 1] } else { z };
                    acc[j][k] = _mm512_alignr_epi64::<1>(next, acc[j][k]);
                }
                acc[j][0] = _mm512_add_epi64(acc[j][0], cv[j]);
                for k in 0..W {
                    acc[j][k] = _mm512_madd52hi_epu64(acc[j][k], av[j][k], bv[j]);
                }
                for k in z1..W {
                    acc[j][k] = _mm512_madd52hi_epu64(acc[j][k], pv[k], mv[j]);
                }
            }
        }
        for (j, out) in outs.into_iter().enumerate() {
            for k in 0..W {
                // SAFETY: as for the loads.
                unsafe { _mm512_storeu_si512(out.as_mut_ptr().add(8 * k).cast(), acc[j][k]) };
            }
            finish::<W>(out, p);
        }
    }

    /// Register k of a 64-bit-limb input as eight 52-bit lanes.
    ///
    /// # Safety
    /// AVX-512F/BW/VBMI. `x` is read only through a byte mask covering its
    /// own length.
    #[inline]
    #[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]
    unsafe fn load52(x: &[u64], t: &ConvTables, k: usize) -> __m512i {
        let bytes = 8 * x.len();
        let base = t.base[k];
        let avail = bytes.saturating_sub(base).min(64);
        let lmask: u64 = if avail == 64 { u64::MAX } else { (1u64 << avail) - 1 };
        // SAFETY: the mask covers only bytes base..base + avail, inside x;
        // masked-off bytes are not accessed. When avail is 0 nothing loads.
        let raw = unsafe { _mm512_maskz_loadu_epi8(lmask, x.as_ptr().cast::<i8>().wrapping_add(base)) };
        // SAFETY: the tables are 64-byte arrays.
        let (perm, ctrl) = unsafe {
            (
                _mm512_loadu_si512(t.perm[k].as_ptr().cast()),
                _mm512_loadu_si512(t.ctrl[k].as_ptr().cast()),
            )
        };
        let g = _mm512_maskz_permutexvar_epi8(t.keep[k], perm, raw);
        _mm512_and_si512(_mm512_multishift_epi64_epi8(ctrl, g), _mm512_set1_epi64(MASK as i64))
    }

    /// [`mont_mul_k`] with operands read from 64-bit limbs via `load52`.
    ///
    /// # Safety
    /// AVX-512F/BW/IFMA/VBMI; inputs are read only within their lengths and
    /// outputs hold 8 * W words.
    #[target_feature(enable = "avx512f,avx512bw,avx512ifma,avx512vbmi")]
    pub(super) unsafe fn mont_mul64_k<const W: usize, const K: usize>(
        outs: [&mut [u64]; K],
        as_: [&[u64]; K],
        bs: [&[u64]; K],
        ta: &ConvTables,
        tb: &ConvTables,
        p: &[u64],
        p1: &[u64],
        nl: usize,
        z1: usize,
    ) {
        let z = _mm512_setzero_si512();
        let mut av = [[z; W]; K];
        let mut pv = [z; W];
        let mut acc = [[z; W]; K];
        let mut b52 = [[0u64; 128]; K];
        for k in 0..W {
            // SAFETY: k < W and p1 holds 8 * W words; load52 reads within the
            // inputs; b52 rows hold 128 >= 8 * W words.
            unsafe {
                pv[k] = _mm512_loadu_si512(p1.as_ptr().add(8 * k).cast());
                for j in 0..K {
                    av[j][k] = load52(as_[j], ta, k);
                    let bk = load52(bs[j], tb, k);
                    _mm512_storeu_si512(b52[j].as_mut_ptr().add(8 * k).cast(), bk);
                }
            }
        }
        for i in 0..nl {
            let mut bv = [z; K];
            let mut mv = [z; K];
            let mut cv = [z; K];
            for j in 0..K {
                bv[j] = _mm512_set1_epi64(b52[j][i] as i64);
                for k in 0..W {
                    acc[j][k] = _mm512_madd52lo_epu64(acc[j][k], av[j][k], bv[j]);
                }
            }
            for j in 0..K {
                let a0 = _mm_cvtsi128_si64(_mm512_castsi512_si128(acc[j][0])) as u64;
                mv[j] = _mm512_set1_epi64((a0 & MASK) as i64);
                cv[j] = _mm512_maskz_set1_epi64(1, (a0 >> 52) as i64);
            }
            for j in 0..K {
                for k in z1..W {
                    acc[j][k] = _mm512_madd52lo_epu64(acc[j][k], pv[k], mv[j]);
                }
                for k in 0..W {
                    let next = if k + 1 < W { acc[j][k + 1] } else { z };
                    acc[j][k] = _mm512_alignr_epi64::<1>(next, acc[j][k]);
                }
                acc[j][0] = _mm512_add_epi64(acc[j][0], cv[j]);
                for k in 0..W {
                    acc[j][k] = _mm512_madd52hi_epu64(acc[j][k], av[j][k], bv[j]);
                }
                for k in z1..W {
                    acc[j][k] = _mm512_madd52hi_epu64(acc[j][k], pv[k], mv[j]);
                }
            }
        }
        for (j, out) in outs.into_iter().enumerate() {
            for k in 0..W {
                // SAFETY: out holds 8 * W words.
                unsafe { _mm512_storeu_si512(out.as_mut_ptr().add(8 * k).cast(), acc[j][k]) };
            }
            finish::<W>(out, p);
        }
    }

    /// Carry-normalise to 52-bit limbs, then subtract p once under a mask:
    /// the value is below 2p.
    #[inline(always)]
    fn finish<const W: usize>(out: &mut [u64], p: &[u64]) {
        let len = 8 * W;
        let mut c = 0u64;
        for w in out.iter_mut().take(len) {
            let v = *w + c;
            *w = v & MASK;
            c = v >> 52;
        }
        // Compute out - p in place, then add p back under a mask when the
        // subtraction borrowed (out < p).
        let mut borrow = 0u64;
        for i in 0..len {
            let v = out[i].wrapping_sub(p[i]).wrapping_sub(borrow);
            out[i] = v & MASK;
            borrow = v >> 63;
        }
        let back = borrow.wrapping_neg(); // all ones when out was < p
        let mut c = 0u64;
        for i in 0..len {
            let v = out[i] + (p[i] & back) + c;
            out[i] = v & MASK;
            c = v >> 52;
        }
    }
}
