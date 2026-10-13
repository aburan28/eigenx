# Theta_dim4 KaniEndoHalf — smallest successful instance

Run on 2026-10-13 with SageMath 10.10 and
`https://github.com/ThetaIsogenies/Theta_dim4` at `/tmp/Theta_dim4`
(`research/kani4_sage/run_smallest.sh`).

Parameters (`pkg/parameters/parameters_3.txt` index 1; index 0 raises
`NotImplementedError` on a product codomain in tiny characteristic, which
the upstream tests also skip):

| | |
|---|---|
| p | 10319560703 = 2^19 · 3^9 − 1 |
| σ degree | 3^9 |
| (a1, a2, e_A) | (213, 22, 16) with 213² + 22² + 3^9 = 2^16 |
| chain | KaniEndoHalf, length 16, torsion 2^10 |

Result: **evaluation correct (`True`)**. Dimension-4 endomorphism 0.11 s,
point evaluation 0.004 s (one core).

This is the reference library’s own instance, not an Eigen-Séta ciphertext.
It shows the dim-4 evaluator runs. Wiring a Toy16 Eigen-Séta `(pk, ct)` into
`KaniEndoHalf` (same field, masks, basis) is still open — see `attack.sage`.
