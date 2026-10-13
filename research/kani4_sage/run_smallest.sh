#!/bin/sh
# Run the smallest published Theta_dim4 KaniEndoHalf instance (ℓ=3, e_A=16).
# Requires a checkout of https://github.com/ThetaIsogenies/Theta_dim4
# (this repo does not vendor it; it is MIT, see that tree's LICENSE).
#
#   THETA_DIM4=/tmp/Theta_dim4 sh research/kani4_sage/run_smallest.sh
set -eu
ROOT="${THETA_DIM4:-/tmp/Theta_dim4}"
cd "$ROOT/Theta_dim4_sage"
# parameters_3.txt index 1 (index 0 hits a known product-codomain exception
# in small characteristic). e_A=16 e_B=9 a1=213 a2=22 f=1 f_A=19 f_B=9
# p=10319560703. 213^2 + 22^2 + 3^9 = 2^16.
exec sage Tests.py --KaniEndoHalf --no_primality_check \
  -l_B 3 -e_A 16 -e_B 9 -a1 213 -a2 22 -f 1 -f_A 19 -f_B 9 -p 10319560703
