//! Cryptanalysis routines for Eigen-Séta.
//!
//! Nothing in this module is used by the KEM. It exists so that the cost
//! model in `security` can be checked against concrete operations:
//!
//! * `degree`: key recovery from the secret degree f (confirms that the key
//!   is a function of f and public data), and the near-miss experiment that
//!   tests whether a wrong guess of f fails early (which would localise f)
//!   or only at the end of the chain;
//! * `mitm3`: meet-in-the-middle recovery of the sender's 3^F3-isogeny
//!   kernel, the generic message-recovery attack, with its measured cost;
//! * `kani4`: mask-guess + Robert/Kani message-recovery scaffolding (form
//!   kernel, product-isogeny oracle cost; full dim-4 eval in
//!   `research/kani4_sage/`);
//! * `pk_test`: testing a candidate degree on the public key alone, with a
//!   Kani chain of about 2·f_bits steps on E_A × E_A instead of a
//!   decapsulation;
//! * `checks`: structural conditions that known attacks on related schemes
//!   exploit, checked on actual keys.
//!
//! Everything is variable time and handles secret values in the clear.

pub mod checks;
pub mod degree;
pub mod kani4;
pub mod mitm3;
pub mod pk_test;
