# Toy16 Eigen-Séta instance through Theta_dim4

2026-10-13, SageMath 10.10, `THETA_DIM4=/tmp/Theta_dim4`.

```text
cargo run --release --example export_kani4 > research/kani4_sage/toy16_instance.sage
THETA_DIM4=/tmp/Theta_dim4 sage research/kani4_sage/attack_toy16.sage
```

The exporter builds one Toy16 public key (DRBG `kani4-export`), a degree-3^21
isogeny with kernel scalar `s = 1`, and the images of a basis of the
2^23-torsion (`f = ⌈e/2⌉+2`, Robert `e = 41`, `a1 = 466957`, `a2 = 1403750`).
Image y-signs are chosen so `φ(P)+φ(Q)` has the same x-coordinate as `φ(P+Q)`.

Result:

```text
orders 8388608 8388608 8388608 8388608
dim4 0.307s eval 0.022s check True
```

`8388608 = 2^23`. The check is the same x-only identity as Theta_dim4's
`Tests.py`: the dimension-4 endomorphism acts as the Kani matrix on `(P, 0, 0, 0)`.
