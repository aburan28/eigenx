//! Run-time dispatch over the parameter sets.
//!
//! `with_set!(id, P, N, { ... })` expands the block once per parameter set
//! with `P` bound to the `ParamSet` type and `N` to its limb count, and
//! selects the branch matching the `wire::SetId` value `id`.

/// Expand `$body` with `$P`/`$N` bound to the parameter set selected by the
/// `SetId` expression `$id`.
#[macro_export]
macro_rules! with_set {
    ($id:expr, $P:ident, $N:ident, $body:block) => {{
        use $crate::wire::SetId;
        match $id {
            SetId::Toy16 => {
                type $P = $crate::params::Toy16;
                const $N: usize = 4;
                $body
            }
            SetId::Toy32 => {
                type $P = $crate::params::Toy32;
                const $N: usize = 6;
                $body
            }
            SetId::L1 => {
                type $P = $crate::params::L1;
                const $N: usize = 21;
                $body
            }
            SetId::L3 => {
                type $P = $crate::params::L3;
                const $N: usize = 32;
                $body
            }
            SetId::L5 => {
                type $P = $crate::params::L5;
                const $N: usize = 42;
                $body
            }
            SetId::Q3 => {
                type $P = $crate::params::Q3;
                const $N: usize = 61;
                $body
            }
            SetId::Q5 => {
                type $P = $crate::params::Q5;
                const $N: usize = 82;
                $body
            }
        }
    }};
}

/// Summary of a parameter set, computed from its constants.
#[derive(Clone, Debug)]
pub struct SetInfo {
    pub name: &'static str,
    pub lambda: usize,
    pub p_bits: usize,
    pub limbs: usize,
    pub e: usize,
    pub f3: usize,
    pub d_bits: f64,
    pub f_bits: usize,
    pub pk_bytes: usize,
    pub ct_bytes: usize,
    pub sk_bytes: usize,
    pub quantum_hardened: bool,
}

pub fn set_info(id: crate::wire::SetId) -> SetInfo {
    use crate::fp::FpConfig;
    use crate::kem::EigenSeta;
    use crate::params::ParamSet;
    with_set!(id, P, N, {
        let m = <P as ParamSet<N>>::Cfg::MODULUS;
        let mut p_bits = 0;
        for i in (0..N).rev() {
            if m[i] != 0 {
                p_bits = 64 * i + (64 - m[i].leading_zeros() as usize);
                break;
            }
        }
        let (lo, _) = EigenSeta::<P, N>::secret_degree_range();
        let f3 = <P as ParamSet<N>>::F3;
        let lambda = <P as ParamSet<N>>::LAMBDA;
        let d_bits = f3 as f64 * 3f64.log2();
        SetInfo {
            name: <P as ParamSet<N>>::NAME,
            lambda,
            p_bits,
            limbs: N,
            e: <P as ParamSet<N>>::E,
            f3,
            d_bits,
            f_bits: lo.bits() as usize,
            pk_bytes: EigenSeta::<P, N>::pk_bytes(),
            ct_bytes: EigenSeta::<P, N>::ct_bytes(),
            sk_bytes: EigenSeta::<P, N>::sk_bytes(),
            quantum_hardened: d_bits >= 4.0 * lambda as f64,
        }
    })
}
