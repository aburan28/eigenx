//! Runnable cryptanalysis of Eigen-Séta at toy scale.
//!
//! Nothing in this module is used by the KEM.  It exists so that the cost
//! model in `security` rests on experiments, not only on arithmetic, and so
//! that anyone can re-run those experiments:
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
//! * `checks`: structural conditions that known attacks on related schemes
//!   exploit, checked on actual keys.
//!
//! Everything is variable time and handles secret values in the clear.

pub mod checks;
pub mod degree;
pub mod kani4;
pub mod mitm3;
