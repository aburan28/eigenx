# Evaluate Theta_dim4 KaniEndoHalf on one exported Eigen-Seta Toy16 instance.
#
#   cargo run --release --example export_kani4 > research/kani4_sage/toy16_instance.sage
#   THETA_DIM4=/tmp/Theta_dim4 sage research/kani4_sage/attack_toy16.sage
#
# The Rust exporter evaluates the degree-3^F3 isogeny. This file only builds
# the curves and runs the dimension-4 chain.

import os
import sys

from sage.all import GF, ZZ, EllipticCurve, ceil

THETA = os.environ.get("THETA_DIM4", "/tmp/Theta_dim4")
sys.path.insert(0, os.path.join(THETA, "Theta_dim4_sage"))

from pkg.isogenies.Kani_endomorphism import KaniEndoHalf
from pkg.theta_structures.Tuple_point import TuplePoint
from pkg.utilities.strategy import precompute_strategy_with_first_eval

here = os.path.dirname(os.path.abspath(__file__))
inst = os.environ.get("KANI4_INSTANCE", os.path.join(here, "toy16_instance.sage"))
load(inst)

if q + a1**2 + a2**2 != 2**e:
    raise ValueError("exported Robert parameters do not satisfy 2^e = q + a1^2 + a2^2")

Fp2 = GF(p**2, "i", modulus=[1, 0, 1], proof=False)
i = Fp2.gen()


def fe(pair):
    c0, c1 = pair
    return Fp2(ZZ(c0)) + Fp2(ZZ(c1)) * i


def curve(A):
    return EllipticCurve(Fp2, [0, fe(A), 0, 1, 0])


def point(E, pair):
    x, y = pair
    return E(fe(x), fe(y))


E1 = curve(A1)
E2 = curve(A2)
P1p = point(E1, P1)
Q1p = point(E1, Q1)
R2p = point(E2, R2)
S2p = point(E2, S2)

print("orders", P1p.order(), Q1p.order(), R2p.order(), S2p.order())
assert P1p.order() == 2**f
assert R2p.order() == 2**f

m = 0
ai = a2 if a1 % 2 == 1 else a1
while ai % 2 == 0:
    m += 1
    ai //= 2

e1 = ceil(e / 2)
e2 = e - e1
strategy1 = precompute_strategy_with_first_eval(e1, m, M=1, S=0.8, I=100)
strategy2 = strategy1 if e2 == e1 else precompute_strategy_with_first_eval(e2, m, M=1, S=0.8, I=100)

import time

t0 = time.time()
F = KaniEndoHalf(P1p, Q1p, R2p, S2p, q, a1, a2, e, f, strategy1, strategy2)
t1 = time.time()
T = TuplePoint(P1p, E1(0), E2(0), E2(0))
FT = F(T)
t2 = time.time()
# Same x-only check as Theta_dim4 Tests.py (signs of a1/a2 may swap inside).
ok = (FT[0][0] == (a1 * P1p)[0] or FT[0][0] == (a2 * P1p)[0]) and FT[3] == E2(0)
print("dim4 {:.3f}s eval {:.3f}s check {}".format(t1 - t0, t2 - t1, ok))
if not ok:
    print("FT0", FT[0][0])
    print("a1 P", (a1 * P1p)[0])
    print("a2 P", (a2 * P1p)[0])
    raise SystemExit(1)
