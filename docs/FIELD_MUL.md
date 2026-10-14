# Field multiplication

Eigen-Séta primes have the shape `p = 2^(E+1) · 3^F3 · c − 1`, so `p ≡ −1 (mod 2^{64Z})` for a large `Z` (`FpConfig::P1_ZERO_LIMBS`).

`Fp::mul` uses that shape (`mul_friendly`): one schoolbook product, then a Montgomery reduction that skips the `Z` zero limbs of `p+1`. Cost is `N² + N(N−Z)` word products instead of about `2N²` for generic CIOS. The stack buffer covers every set we ship, including Q5 (82 limbs). `tests` in `fp.rs` check the product against integer arithmetic on Toy16, L1, Q3 and Q5.

Gluing in `theta.rs` inverts several independent field elements with one inversion (`batch_invert`).

## What we are not generating

Per-prime Fiat-crypto or hand assembly would emit the same friendly reduction for this prime shape, specialised to one `N`. The crate is const-generic over seven NUMS primes (`docs/NUMS.md`). A codegen pipeline would have to be re-run whenever the recipe changes, and it would not change the algorithm. We keep the generic friendly reduction, which is what those generators emit, and we test it.

Assembly is worth it only after a profile shows the Rust word-mul loop, rather than the isogeny strategy, dominating L5/Q5 decapsulation.
