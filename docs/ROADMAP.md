# EigenX remaining work: PQ claims, review, and performance

This is the checklist for the four priorities from the security/performance
assessment. Status key: `[x]` done in-repo, `[~]` started, `[ ]` still open
(needs people, Sage, or longer engineering).

## 1. Two claim tiers (never market L1 as quantum Cat-1 under the query bar)

| Item | Status |
|---|---|
| Document **Tier A** (NIST gate-count / MAXDEPTH): L1 / L3 / L5 → Cat 1 / 3 / 5 | [x] README + SECURITY §5.3 |
| Document **Tier B** (≥ 2^λ quantum queries): L5 (=Q1) / Q3 / Q5 | [x] README + SECURITY §5.3 |
| Explicit “do not claim” table for L1/L3 quantum queries | [x] README deployment map |
| CLI `params` shows claim-tier columns | [x] `eigenx params` |
| Accept `Q1` as an alias for `L5` | [x] `SetId::from_name` |

Still open outside this repository:

| Item | Status |
|---|---|
| External sign-off that the constant 2^8 in the Grover oracle is defensible | [ ] reviewers on PR #1. The formula and the Toy16 measurement are in `security.rs` and `LITERATURE_DRILL.md` L3 |

## 2. Recommend L5 / Q3 / Q5 for post-quantum deployments

| Item | Status |
|---|---|
| Recommended PQ suite spelled out (L5, Q3, Q5) | [x] README |
| Classical-optimized suite (L1, L3) with quantum caveat | [x] README |
| Example CLI snippets defaulting PQ examples to L5 | [x] README |
| CI still exercises L1 (speed) plus cost-model assertions for Q floors | [x] existing CI |
| Timed Tier-B round trip | [x] Q5: keygen 9825 s (8 jobs), encaps 93 s, decaps 214 s (`research/q5_roundtrip.md`) |
| PQ entry point refusing L1/L3 | [x] `scripts/pq_roundtrip.sh` |

## 3. External review + kani4_sage + level-structure literature

| Item | Status |
|---|---|
| PR #1 open with REVIEW.md reproduction table | [x] |
| Reviewer checklist for Tier A vs Tier B claims | [x] docs/REVIEW.md |
| Open questions C3/C6/C7/C8 called out for reviewers | [x] SECURITY §8.1 |
| Rust `cryptanalysis::kani4` scaffolding + CI | [x] |
| Sage driver stub + setup instructions | [x] research/kani4_sage/ |
| Literature drill template (DFP / Arpin / Macula–Stange) | [x] docs/LITERATURE_DRILL.md |
| Run Theta_dim4 `KaniEndoHalf` on the published e_A=16 instance (eval correct) | [x] `research/kani4_sage/SMALLEST_RUN.md` |
| Recover an Eigen-Séta Toy16 isogeny image inside Theta_dim4 | [x] `attack_toy16.sage` (eval check true, 0.3 s) |
| Record that measurement on the mask-guess row | [x] `security.rs` note |
| Author literature drill | [x] `docs/LITERATURE_DRILL.md` (L2 mask-free, L6, L8 still need an outside cryptanalyst) |
| FO writeup checked against `kem.rs`; δ_corr probed at Toy16 | [x] `FO_PROOF.md`, `toy16_recovers_several_kernel_scalars` |
| External peer review of the FO proof, and a proof that δ_corr = 0 | [ ] not something this repository can close |
| Cryptanalysis-finding issues on PR #1 | [ ] none filed yet; the template is in `.github/ISSUE_TEMPLATE` |

## 4. Performance: field/asm and keygen parallelism

| Item | Status |
|---|---|
| Theta (2,2)-chain in KEM decaps | [x] |
| Parallel keygen attempts (`keygen_with_jobs`, CLI `--jobs`) | [x] |
| Re-tune attacker oracle costs after defender theta (document margin) | [x] gate model stays attacker-cheap; Toy16 wall clock is only a note |
| NUMS primes: smallest c, fixed MR bases, `verify_nums` | [x] `docs/NUMS.md` |
| Montgomery-friendly mul for Q3/Q5 (limb cap 96) | [x] `fp::mul_friendly`, `docs/FIELD_MUL.md` |
| Fiat-crypto / assembly codegen | [x] declined in `docs/FIELD_MUL.md`: same algorithm, seven NUMS primes |
| Batch inversion in theta gluing | [x] `theta::batch_invert` |
| Parallel keygen | [x] `keygen_with_jobs`. One attempt is a single Cornacchia search on one RNG; independent attempts are the parallel unit |
| Fault threat model | [x] `SIDE_CHANNELS.md` §7. Countermeasures beyond FO re-encryption are not implemented |

## Definition of “fully working / meeting criteria”

For this repository’s own claims:

1. **Tier A met** when `tests/security.rs` stays green for L1/L3/L5 gate floors
   (already true) and README/CLI never imply Tier B for L1/L3.
2. **Tier B met** when Q3/Q5 (and L5 as Q1) stay green for quantum-query ≥ λ
   (already true) and are the documented PQ defaults.
3. **Review-ready** when PR #1 has clear tier language and a filled literature
   drill *or* explicit “unchecked” rows — not when every open question is
   closed (that is research).
4. **Performance-usable** when Toy16/L1 remain interactive and Q* keygen can
   use `--jobs` to cut wall time roughly by the number of cores (rejection
   sampling).

In-repo work on these four priorities is done. What remains is outside the
repository: a reviewer’s sign-off on the Grover-oracle constant, a proof that
`δ_corr = 0`, and cryptanalysis of the three rows marked `needs-experiment`
in `docs/LITERATURE_DRILL.md` (mask-free higher-dimensional embedding, level
structure, sesquilinear pairings).
