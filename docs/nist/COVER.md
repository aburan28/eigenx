# Cover note (draft)

**Not a submission. Not ready to upload. Not signed.**

Algorithm name: Eigen-Séta (implementation name EigenX), version 2, the
secret-degree isogeny KEM in this repository.

Algorithm type: key-encapsulation mechanism. An optional KEM-DEM public-key
encryption wrapper exists in `src/pke.rs`. The claims below are about the
KEM. The PKE wrapper is not given a separate category.

Principal submitter, auxiliary submitters, inventors' affidavit, owner, and
backup point of contact: **not filled in.** The code copyright line in
`LICENSE` is not a NIST cover sheet.

## The only category statement this draft is willing to make

**A submission that recommends L1 or L3 as NIST Category 1 or 3 is a false
submission.**

L1 meets a NIST gate-count style Tier A argument only. It is not NIST
Category 1 under a 2^λ query bar. The binding quantum attack is Grover on
mask-guess, about 2^{λ/2} queries. A submission that recommends L1 as NIST
Category 1 is a false submission.

L3 meets a NIST gate-count style Tier A argument only. It is not NIST
Category 3 under a 2^λ query bar. The binding quantum attack is Grover on
mask-guess, about 2^{λ/2} queries. A submission that recommends L3 as NIST
Category 3 is a false submission.

L5 is claimed at ≥ 2^128 quantum queries and ≥ 2^256 classical operations:
quantum category 1 and classical category 5. The binding quantum attack is
Grover on mask-guess; the cost model reports about 2^{128.5} queries. L5 is
not a quantum category 5 set. There is no separate Q1 prime: the
quantum-hardened level-1 parameters are L5.

Q3 is claimed as category 3: ≥ 2^192 quantum queries against every known
attack in the cost model. Q5 is claimed as category 5: ≥ 2^256 quantum
queries against every known attack in the cost model.

Those floors are calculations in `src/security.rs`, asserted by
`tests/security.rs`, under the assumptions in `docs/SECURITY.md`. They are
not a security proof.

## What is not claimed

- QROM IND-CCA is not claimed for any set.
- The FO writeup in `docs/FO_PROOF.md` is author-checked against `src/kem.rs`.
  It has not been externally reviewed.
- Eigen-CIST and secret-degree EndRing are assumptions, not reductions to a
  standard problem.
- No patent is claimed here. No patent search has been done. See the
  intellectual-property placeholder in the specification.

Parameter primes for L1, L3, L5, Q3, and Q5 are the nothing-up-my-sleeve
primes already in `src/params_generated.rs`. This draft does not change them.
Toy16 and Toy32 are not parameter sets of this package.
