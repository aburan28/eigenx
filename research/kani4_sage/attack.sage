# Dimension-4 Kani message recovery driver (stub).
#
# Wire a Toy16 (pk, ct) into Theta_dim4's Kani_endomorphism once that
# library is on SAGE_PATH.  The Rust module src/cryptanalysis/kani4.rs
# already builds form_kernel and times the dim-2 product-isogeny oracle;
# this file is the missing dim-4 evaluation.
#
# Usage (once Theta_dim4 is installed):
#   sage -c "load('attack.sage'); run_from_hex('<pk_hex>', '<ct_hex>')"
#
# See README.md.

import os
import sys

THETA = os.environ.get("THETA_DIM4")
if THETA:
    sys.path.insert(0, THETA)


def robert_params(d, e_max=None):
    """Smallest e with 2^e = d + a1^2 + a2^2 and a1 odd."""
    from sage.all import ZZ, two_squares  # noqa: F401 — Sage runtime

    d = ZZ(d)
    lo = (d.nbits() * ZZ(3).log(2)).ceil()
    hi = e_max or (lo + 64)
    for e in range(int(lo), int(hi) + 1):
        m = ZZ(2) ** e - d
        if m <= 0:
            continue
        try:
            # Sage's sum_of_two_squares / manual search
            for a1 in range(1, int(m.sqrt()) + 1, 2):
                rest = m - a1 * a1
                a2 = ZZ(rest).isqrt()
                if a2 * a2 == rest:
                    return e, ZZ(a1), ZZ(a2)
        except Exception:
            continue
    raise ValueError("no Robert parameters")


def e_mask_for(e):
    return (e + 1) // 2 + 2


def run_from_hex(pk_hex, ct_hex):
    """
    Placeholder entry point.

    Steps once Theta_dim4 is available:
      1. Parse Eigen-Séta pk/ct field elements (4 Fp2 each).
      2. Enumerate odd (a,b) mod 2^{e_mask} (or a known pair).
      3. Unmask torsion images; build form_kernel(a1,a2,...).
      4. Evaluate Kani_endomorphism via Theta_dim4.
      5. Read off the action of φ on the 3-torsion basis → s.
      6. Confirm by re-encryption (Rust encrypt_scalar).
    """
    raise NotImplementedError(
        "Clone https://github.com/ThetaIsogenies/Theta_dim4, set THETA_DIM4, "
        "and implement steps 1–6 against Kani_endomorphism.py. "
        "Rust CI covers form_kernel + oracle cost without this."
    )


if __name__ == "__main__":
    print("kani4_sage stub: see README.md and run_from_hex()")
