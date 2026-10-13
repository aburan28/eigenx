# Dimension-4 Kani message recovery (Sage)

The Rust module `src/cryptanalysis/kani4.rs` implements the **mask-guess
scaffolding** of the Robert / KaniEndo message-recovery attack:

* Robert parameters `(e, a1, a2)` with `2^e = d + a1² + a2²`;
* mask precision `e_mask = ⌈e/2⌉ + 2` (KaniEndoHalf);
* `form_kernel` generators on `E_A² × E_C²`;
* a dim-2 `product_isogeny` oracle as the per-guess cost model;
* recovery on a **known honest mask** via the existing MITM (stand-in).

Full evaluation of the endomorphism

```text
F = [[α,  diag(ˆφ)],
     [-diag(φ), ˆα]]  ∈ End(E_A² × E_C²)
```

needs **dimension-4 level-2 theta isogenies** (ePrint 2024/1180). That code
lives in the [Theta_dim4](https://github.com/ThetaIsogenies/Theta_dim4)
reference (Sage). This directory is the driver that wires Eigen-Séta toy
ciphertexts into that library.

## Layout

| File | Role |
|---|---|
| `README.md` | this note |
| `attack.sage` | load a Toy16 `(pk, ct)` dump, guess / apply masks, call Theta_dim4 `Kani_endomorphism`, recover `s` |
| `export_toy.sage` | helper: read hex dumps written by `cargo run --example cryptanalysis -- …` |

## Prerequisites

1. SageMath ≥ 9.5 with Theta_dim4 on `SAGE_PATH` (clone next to this tree or
   set `THETA_DIM4`).
2. A Toy16 public key and ciphertext as hex (from the Rust example or KATs).

## Expected behaviour

At Toy16 (`d = 3^21`, Robert `e ≈ 41`, `e_mask ≈ 23`) a **correct** mask
pair yields `s` in seconds once Theta_dim4 is available. Wrong masks fail
the dim-4 evaluation (or produce a value that fails re-encryption). The
Rust CI asserts the cost-model shape and the honest-mask recovery path
without requiring Sage.

## Status

Driver stub: the Rust side is what CI runs. Fill in `attack.sage` against
a local Theta_dim4 checkout when reproducing the full poly-time recovery.
