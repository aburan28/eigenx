#[inline(never)]
pub fn prod1(a0: &[u64; 21], b0: &[u64; 21], lo0: &mut [u64; 21], hi0: &mut [u64; 21]) {
    let mut acc0: u128 = 0;
    let mut e0: u64 = 0;
    { let p = (a0[0] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[0] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    { let p = (a0[0] as u128) * (b0[1] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[1] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[1] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    { let p = (a0[0] as u128) * (b0[2] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[1] as u128) * (b0[1] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[2] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[2] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[3] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[2] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[1] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[0] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    lo0[3] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[4] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[3] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[2] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[1] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[4] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[4] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[5] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[4] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[3] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[2] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[4] as u128) * (b0[1] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[5] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[5] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[6] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[5] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[4] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[3] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[4] as u128) * (b0[2] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[5] as u128) * (b0[1] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[6] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[6] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[7] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[6] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[5] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[4] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[3] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[2] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[1] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[0] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    lo0[7] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[8] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[7] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[6] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[5] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[4] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[3] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[2] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[1] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[8] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[8] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[9] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[8] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[7] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[6] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[5] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[4] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[3] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[2] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[8] as u128) * (b0[1] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[9] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[9] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[10] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[9] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[8] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[7] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[6] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[5] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[4] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[3] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[8] as u128) * (b0[2] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[9] as u128) * (b0[1] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[10] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[10] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[11] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[10] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[9] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[8] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[7] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[6] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[5] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[4] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[8] as u128) * (b0[3] as u128);
    let q0_1: u128 = (a0[9] as u128) * (b0[2] as u128);
    let q0_2: u128 = (a0[10] as u128) * (b0[1] as u128);
    let q0_3: u128 = (a0[11] as u128) * (b0[0] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    lo0[11] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[12] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[11] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[10] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[9] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[8] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[7] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[6] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[5] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[8] as u128) * (b0[4] as u128);
    let q0_1: u128 = (a0[9] as u128) * (b0[3] as u128);
    let q0_2: u128 = (a0[10] as u128) * (b0[2] as u128);
    let q0_3: u128 = (a0[11] as u128) * (b0[1] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[12] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[12] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[13] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[12] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[11] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[10] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[9] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[8] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[7] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[6] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[8] as u128) * (b0[5] as u128);
    let q0_1: u128 = (a0[9] as u128) * (b0[4] as u128);
    let q0_2: u128 = (a0[10] as u128) * (b0[3] as u128);
    let q0_3: u128 = (a0[11] as u128) * (b0[2] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[12] as u128) * (b0[1] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[13] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[13] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[14] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[13] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[12] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[11] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[10] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[9] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[8] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[7] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[8] as u128) * (b0[6] as u128);
    let q0_1: u128 = (a0[9] as u128) * (b0[5] as u128);
    let q0_2: u128 = (a0[10] as u128) * (b0[4] as u128);
    let q0_3: u128 = (a0[11] as u128) * (b0[3] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[12] as u128) * (b0[2] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[13] as u128) * (b0[1] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[14] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[14] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[15] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[14] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[13] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[12] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[11] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[10] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[9] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[8] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[8] as u128) * (b0[7] as u128);
    let q0_1: u128 = (a0[9] as u128) * (b0[6] as u128);
    let q0_2: u128 = (a0[10] as u128) * (b0[5] as u128);
    let q0_3: u128 = (a0[11] as u128) * (b0[4] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[12] as u128) * (b0[3] as u128);
    let q0_1: u128 = (a0[13] as u128) * (b0[2] as u128);
    let q0_2: u128 = (a0[14] as u128) * (b0[1] as u128);
    let q0_3: u128 = (a0[15] as u128) * (b0[0] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    lo0[15] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[12] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[11] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[10] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[9] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[8] as u128) * (b0[8] as u128);
    let q0_1: u128 = (a0[9] as u128) * (b0[7] as u128);
    let q0_2: u128 = (a0[10] as u128) * (b0[6] as u128);
    let q0_3: u128 = (a0[11] as u128) * (b0[5] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[12] as u128) * (b0[4] as u128);
    let q0_1: u128 = (a0[13] as u128) * (b0[3] as u128);
    let q0_2: u128 = (a0[14] as u128) * (b0[2] as u128);
    let q0_3: u128 = (a0[15] as u128) * (b0[1] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[16] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[16] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[17] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[16] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[15] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[14] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[13] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[12] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[11] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[10] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[8] as u128) * (b0[9] as u128);
    let q0_1: u128 = (a0[9] as u128) * (b0[8] as u128);
    let q0_2: u128 = (a0[10] as u128) * (b0[7] as u128);
    let q0_3: u128 = (a0[11] as u128) * (b0[6] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[12] as u128) * (b0[5] as u128);
    let q0_1: u128 = (a0[13] as u128) * (b0[4] as u128);
    let q0_2: u128 = (a0[14] as u128) * (b0[3] as u128);
    let q0_3: u128 = (a0[15] as u128) * (b0[2] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[16] as u128) * (b0[1] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[17] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[17] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[18] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[17] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[16] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[15] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[14] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[13] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[12] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[11] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[8] as u128) * (b0[10] as u128);
    let q0_1: u128 = (a0[9] as u128) * (b0[9] as u128);
    let q0_2: u128 = (a0[10] as u128) * (b0[8] as u128);
    let q0_3: u128 = (a0[11] as u128) * (b0[7] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[12] as u128) * (b0[6] as u128);
    let q0_1: u128 = (a0[13] as u128) * (b0[5] as u128);
    let q0_2: u128 = (a0[14] as u128) * (b0[4] as u128);
    let q0_3: u128 = (a0[15] as u128) * (b0[3] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[16] as u128) * (b0[2] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[17] as u128) * (b0[1] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[18] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[18] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[19] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[18] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[17] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[16] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[15] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[14] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[13] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[12] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[8] as u128) * (b0[11] as u128);
    let q0_1: u128 = (a0[9] as u128) * (b0[10] as u128);
    let q0_2: u128 = (a0[10] as u128) * (b0[9] as u128);
    let q0_3: u128 = (a0[11] as u128) * (b0[8] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[12] as u128) * (b0[7] as u128);
    let q0_1: u128 = (a0[13] as u128) * (b0[6] as u128);
    let q0_2: u128 = (a0[14] as u128) * (b0[5] as u128);
    let q0_3: u128 = (a0[15] as u128) * (b0[4] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[16] as u128) * (b0[3] as u128);
    let q0_1: u128 = (a0[17] as u128) * (b0[2] as u128);
    let q0_2: u128 = (a0[18] as u128) * (b0[1] as u128);
    let q0_3: u128 = (a0[19] as u128) * (b0[0] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    lo0[19] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[0] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[1] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[2] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[3] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[4] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[8] as u128) * (b0[12] as u128);
    let q0_1: u128 = (a0[9] as u128) * (b0[11] as u128);
    let q0_2: u128 = (a0[10] as u128) * (b0[10] as u128);
    let q0_3: u128 = (a0[11] as u128) * (b0[9] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[12] as u128) * (b0[8] as u128);
    let q0_1: u128 = (a0[13] as u128) * (b0[7] as u128);
    let q0_2: u128 = (a0[14] as u128) * (b0[6] as u128);
    let q0_3: u128 = (a0[15] as u128) * (b0[5] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[16] as u128) * (b0[4] as u128);
    let q0_1: u128 = (a0[17] as u128) * (b0[3] as u128);
    let q0_2: u128 = (a0[18] as u128) * (b0[2] as u128);
    let q0_3: u128 = (a0[19] as u128) * (b0[1] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[20] as u128) * (b0[0] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    lo0[20] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[1] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[2] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[3] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[4] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[5] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[6] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[7] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[8] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[9] as u128) * (b0[12] as u128);
    let q0_1: u128 = (a0[10] as u128) * (b0[11] as u128);
    let q0_2: u128 = (a0[11] as u128) * (b0[10] as u128);
    let q0_3: u128 = (a0[12] as u128) * (b0[9] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[13] as u128) * (b0[8] as u128);
    let q0_1: u128 = (a0[14] as u128) * (b0[7] as u128);
    let q0_2: u128 = (a0[15] as u128) * (b0[6] as u128);
    let q0_3: u128 = (a0[16] as u128) * (b0[5] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[17] as u128) * (b0[4] as u128);
    let q0_1: u128 = (a0[18] as u128) * (b0[3] as u128);
    let q0_2: u128 = (a0[19] as u128) * (b0[2] as u128);
    let q0_3: u128 = (a0[20] as u128) * (b0[1] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    hi0[0] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[2] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[3] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[4] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[5] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[6] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[7] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[8] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[9] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[10] as u128) * (b0[12] as u128);
    let q0_1: u128 = (a0[11] as u128) * (b0[11] as u128);
    let q0_2: u128 = (a0[12] as u128) * (b0[10] as u128);
    let q0_3: u128 = (a0[13] as u128) * (b0[9] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[14] as u128) * (b0[8] as u128);
    let q0_1: u128 = (a0[15] as u128) * (b0[7] as u128);
    let q0_2: u128 = (a0[16] as u128) * (b0[6] as u128);
    let q0_3: u128 = (a0[17] as u128) * (b0[5] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[18] as u128) * (b0[4] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[19] as u128) * (b0[3] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[20] as u128) * (b0[2] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[1] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[3] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[4] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[5] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[6] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[7] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[8] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[9] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[10] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[11] as u128) * (b0[12] as u128);
    let q0_1: u128 = (a0[12] as u128) * (b0[11] as u128);
    let q0_2: u128 = (a0[13] as u128) * (b0[10] as u128);
    let q0_3: u128 = (a0[14] as u128) * (b0[9] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[15] as u128) * (b0[8] as u128);
    let q0_1: u128 = (a0[16] as u128) * (b0[7] as u128);
    let q0_2: u128 = (a0[17] as u128) * (b0[6] as u128);
    let q0_3: u128 = (a0[18] as u128) * (b0[5] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[19] as u128) * (b0[4] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[20] as u128) * (b0[3] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[2] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[4] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[5] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[6] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[7] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[8] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[9] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[10] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[11] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[12] as u128) * (b0[12] as u128);
    let q0_1: u128 = (a0[13] as u128) * (b0[11] as u128);
    let q0_2: u128 = (a0[14] as u128) * (b0[10] as u128);
    let q0_3: u128 = (a0[15] as u128) * (b0[9] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[16] as u128) * (b0[8] as u128);
    let q0_1: u128 = (a0[17] as u128) * (b0[7] as u128);
    let q0_2: u128 = (a0[18] as u128) * (b0[6] as u128);
    let q0_3: u128 = (a0[19] as u128) * (b0[5] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[20] as u128) * (b0[4] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[3] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[5] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[6] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[7] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[8] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[9] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[10] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[11] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[12] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[13] as u128) * (b0[12] as u128);
    let q0_1: u128 = (a0[14] as u128) * (b0[11] as u128);
    let q0_2: u128 = (a0[15] as u128) * (b0[10] as u128);
    let q0_3: u128 = (a0[16] as u128) * (b0[9] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[17] as u128) * (b0[8] as u128);
    let q0_1: u128 = (a0[18] as u128) * (b0[7] as u128);
    let q0_2: u128 = (a0[19] as u128) * (b0[6] as u128);
    let q0_3: u128 = (a0[20] as u128) * (b0[5] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    hi0[4] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[6] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[7] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[8] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[9] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[10] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[11] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[12] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[13] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[14] as u128) * (b0[12] as u128);
    let q0_1: u128 = (a0[15] as u128) * (b0[11] as u128);
    let q0_2: u128 = (a0[16] as u128) * (b0[10] as u128);
    let q0_3: u128 = (a0[17] as u128) * (b0[9] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[18] as u128) * (b0[8] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[19] as u128) * (b0[7] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[20] as u128) * (b0[6] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[5] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[7] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[8] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[9] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[10] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[11] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[12] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[13] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[14] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[15] as u128) * (b0[12] as u128);
    let q0_1: u128 = (a0[16] as u128) * (b0[11] as u128);
    let q0_2: u128 = (a0[17] as u128) * (b0[10] as u128);
    let q0_3: u128 = (a0[18] as u128) * (b0[9] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[19] as u128) * (b0[8] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[20] as u128) * (b0[7] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[6] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[8] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[9] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[10] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[11] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[12] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[13] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[14] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[15] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[16] as u128) * (b0[12] as u128);
    let q0_1: u128 = (a0[17] as u128) * (b0[11] as u128);
    let q0_2: u128 = (a0[18] as u128) * (b0[10] as u128);
    let q0_3: u128 = (a0[19] as u128) * (b0[9] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[20] as u128) * (b0[8] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[7] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[9] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[10] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[11] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[12] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[13] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[14] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[15] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[16] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[17] as u128) * (b0[12] as u128);
    let q0_1: u128 = (a0[18] as u128) * (b0[11] as u128);
    let q0_2: u128 = (a0[19] as u128) * (b0[10] as u128);
    let q0_3: u128 = (a0[20] as u128) * (b0[9] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    hi0[8] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[10] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[11] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[12] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[13] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[14] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[15] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[16] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[17] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[18] as u128) * (b0[12] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[19] as u128) * (b0[11] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[20] as u128) * (b0[10] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[9] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[11] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[12] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[13] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[14] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[15] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[16] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[17] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[18] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[19] as u128) * (b0[12] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[20] as u128) * (b0[11] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[10] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[12] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[13] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[14] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[15] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[16] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[17] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[18] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[19] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[20] as u128) * (b0[12] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[11] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[13] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[14] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[15] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[16] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    let q0_0: u128 = (a0[17] as u128) * (b0[16] as u128);
    let q0_1: u128 = (a0[18] as u128) * (b0[15] as u128);
    let q0_2: u128 = (a0[19] as u128) * (b0[14] as u128);
    let q0_3: u128 = (a0[20] as u128) * (b0[13] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    hi0[12] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[14] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[15] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[16] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[17] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[18] as u128) * (b0[16] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[19] as u128) * (b0[15] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[20] as u128) * (b0[14] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[13] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[15] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[16] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[17] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[18] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[19] as u128) * (b0[16] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[20] as u128) * (b0[15] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[14] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[16] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[17] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[18] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[19] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    { let p = (a0[20] as u128) * (b0[16] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[15] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    let q0_0: u128 = (a0[17] as u128) * (b0[20] as u128);
    let q0_1: u128 = (a0[18] as u128) * (b0[19] as u128);
    let q0_2: u128 = (a0[19] as u128) * (b0[18] as u128);
    let q0_3: u128 = (a0[20] as u128) * (b0[17] as u128);
    { let (s0, o0) = q0_0.overflowing_add(q0_1); let (s1, o1) = q0_2.overflowing_add(q0_3); let (s2, o2) = s0.overflowing_add(s1); let (s3, o3) = acc0.overflowing_add(s2); acc0 = s3; e0 += u64::from(o0) + u64::from(o1) + u64::from(o2) + u64::from(o3); }
    hi0[16] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    { let p = (a0[18] as u128) * (b0[20] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[19] as u128) * (b0[19] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[20] as u128) * (b0[18] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[17] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    { let p = (a0[19] as u128) * (b0[20] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    { let p = (a0[20] as u128) * (b0[19] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[18] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    { let p = (a0[20] as u128) * (b0[20] as u128); let (s, o) = acc0.overflowing_add(p); acc0 = s; e0 += u64::from(o); }
    hi0[19] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
    let mut e0: u64 = 0;
    hi0[20] = acc0 as u64;
    acc0 = (acc0 >> 64) + ((e0 as u128) << 64);
}

