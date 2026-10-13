# Literature drill: open quantum / structural questions

Fill each row after reading the cited paper *against Eigen-Séta’s public
key and ciphertext shapes*. Status: `unchecked` | `no-attack` | `attack`
| `needs-experiment`. Update `docs/SECURITY.md` §8 when a row leaves
`unchecked`.

| ID | Question (SECURITY §8.1) | Citation | Status | Note (one paragraph) |
|---|---|---|---|---|
| L1 | Eigenbasis leakage / hidden orientation | Wesolowski 2021/1583; O-Uber | unchecked | |
| L2 | Mask-free dim-4/8 Kani from subgroups φ⟨P⟩, φ⟨Q⟩ | Robert 2022/1038; ePrint 2024/1180 | unchecked | Rust scaffolding: `cryptanalysis::kani4`; Sage: `research/kani4_sage/` |
| L3 | Grover oracle / MAXDEPTH accounting | NIST call; Jaques–Schanck | unchecked | |
| L4 | Degree-search pruning beyond sum-of-two-squares | — | unchecked | |
| L5 | Composite / smooth f weak keys | SCALLOP / PEARL-SCALLOP | unchecked | |
| L6 | Level structure (eigenbasis as level) | De Feo–Fouotsa–Panny 2024/459; Arpin et al. 2024/1172 | unchecked | |
| L7 | Decisional eigen-CIST well-posed? | design note §5.2 | unchecked | |
| L8 | Sesquilinear / pairing attacks | Macula–Stange (as cited in SECURITY §8) | unchecked | |
| L9 | FO correctness error δ_corr | HHK; FO_PROOF.md | unchecked | |

## How to update a row

1. Reproduce the paper’s attack setting in one sentence.
2. Map each public object in Eigen-Séta to an object the paper needs (or
   mark “missing”).
3. If the attack needs an effective orientation or unmasked torsion the
   scheme does not give, mark `no-attack` and explain.
4. If it might work, sketch the reduction and mark `needs-experiment` or
   `attack` with a cost estimate for L1.

## Dim-4 experiment gate

Before closing L2 as `no-attack`, run:

```text
# after cloning https://github.com/ThetaIsogenies/Theta_dim4
export THETA_DIM4=/path/to/Theta_dim4
sage research/kani4_sage/attack.sage   # see that directory’s README
```

CI does **not** require Sage; it only checks the Rust cost-model shape.
