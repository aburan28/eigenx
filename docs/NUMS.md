# Nothing-up-my-sleeve parameters

Every Eigen-Séta prime is the **unique** output of a public search. There is
no hidden seed, no skipped cofactor, and no Miller–Rabin witness stream that
depends on earlier candidates.

## Recipe

Inputs (all public, listed in `examples/gen_params.rs`):

| Input | Meaning |
|---|---|
| `λ` | security level (16, 32, 128, 192, 256) |
| `m` | `2` for L/Toy (classical), `4` for Q (quantum-query) |
| `κ` | `8` (toys) or `12` (L/Q): low bits of the secret-degree range |

Derived, in order, with no choices:

1. `F3 = ceil(m · λ / log2(3))`, so `d = 3^F3 ≥ 2^(m λ)`.
2. `E` is the **smallest** integer ≥ 8 such that the chain fits for the
   prime found in step 3: `2^E > 28 · d² · f_max²` with
   `f_lo = 2^κ · floor(p / 2^E) + 1` and `f_max = 2 · f_lo`.
3. `c` is the **smallest** positive integer such that
   `p = 2^(E+1) · 3^F3 · c − 1` is prime under [`is_nums_prime`](../src/ntheory.rs)
   **and** `p ≡ 3, 5 or 6 (mod 7)` (so the CM(−7) curve `j = −3375` is
   supersingular).
4. `E_D` coefficients are the reduction of the standard short model of
   `j = −3375`. No random curve.

`is_nums_prime` is trial division by the first 54 primes, then Miller–Rabin
to the **fixed** bases

```text
2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37
```

No random bases. The same integer is accepted or rejected on every machine.

## What this rules out

* A cofactor chosen so that `p − 1` has a secret smooth factor beyond the
  public `2^(E+1) · 3^F3` (any smaller `c` already failed the public test).
* A prime that is only “prime” for one RNG stream of witnesses.
* A non-minimal `E` that hides a shorter chain.

It does **not** prove primality in the formal sense for 5000-bit numbers
(Miller–Rabin with 12 bases is overwhelming but not a deterministic theorem
for every size). Anyone can re-run `examples/verify_nums.rs`. A composite
that slipped through would be a bug to fix by extending `NUMS_MR_BASES`, not
a backdoor.

## Check

```text
cargo run --release --example verify_nums -- Toy16 Toy32
cargo test --release --test nums
```

`verify_nums` checks that the checked-in `COFACTOR` is exactly that minimal
`c`, and that `p` matches `ParamSet`'s modulus.
