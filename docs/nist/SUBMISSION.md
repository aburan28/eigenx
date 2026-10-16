# Eigen-Séta draft submission note

**Draft only. Not uploaded. Not signed. Not externally reviewed.**

Algorithm: Eigen-Séta v2 (crate `eigen_seta`, binary `eigenx`). KEM only.
The optional KEM-DEM wrapper in `src/pke.rs` has no category. Parameter
sets are L1, L3, L5, Q3, and Q5. There is no separate Q1 prime: Q1 is L5
(`src/wire.rs`). Toy16 and Toy32 are not sets in this package. Primes are
the nothing-up-my-sleeve values already in `src/params_generated.rs`.

## Files in this draft

| File | Role |
|---|---|
| `docs/nist/SUBMISSION.md` | This note and the claim sentences |
| `docs/nist/SPEC.md` | Keygen, encaps, decaps, sizes, self-test, implicit rejection |
| `docs/nist/RATIONALE.md` | Design, advantages, limitations, IP placeholder |
| `docs/nist/KAT.md` | How to build a deterministic L1 known-answer file |
| `docs/nist/COVER.md` | Cover placeholder already in the tree |
| `docs/nist/SPECIFICATION.md` | Longer specification already in the tree |
| `docs/nist/GAPS.md` | Open items already in the tree |
| `docs/nist/README.md` | Directory index already in the tree |
| `docs/nist/kat/` | NIST-style L1 vectors, if a short run was produced |
| `src/kem.rs`, `src/wire.rs`, `src/params_generated.rs` | Normative behaviour this draft was read from |
| `docs/SECURITY.md`, `docs/FO_PROOF.md` | Cost model and the author-checked FO note |

## Claim sentences

L1 meets a NIST gate-count style Tier A argument only. It is not NIST
Category 1 under a 2^λ quantum-query bar. The binding quantum attack is
Grover on mask-guess, about 2^{λ/2} queries. A submission that recommends
L1 as NIST Category 1 is a false submission.

L3 meets a NIST gate-count style Tier A argument only. It is not NIST
Category 3 under a 2^λ quantum-query bar. The binding quantum attack is
Grover on mask-guess, about 2^{λ/2} queries. A submission that recommends
L3 as NIST Category 3 is a false submission.

L5 is quantum Category 1 and classical Category 5: at least 2^128 quantum
queries and at least 2^256 classical operations in the cost model. Set Q1
is this set. L5 is not a quantum Category 5 set.

Q3 is quantum Category 3: at least 2^192 quantum queries against every
known attack in the cost model.

Q5 is quantum Category 5: at least 2^256 quantum queries against every
known attack in the cost model.

QROM IND-CCA is not claimed. Eigen-CIST and secret-degree EndRing are
assumptions. `docs/FO_PROOF.md` is author-checked against `src/kem.rs`,
not externally reviewed.
