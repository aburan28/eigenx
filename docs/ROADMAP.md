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

Still open outside docs:

| Item | Status |
|---|---|
| External sign-off that Tier A oracle costing is defensible (SECURITY §8.1 Q3) | [ ] reviewers on PR #1 |

## 2. Recommend L5 / Q3 / Q5 for post-quantum deployments

| Item | Status |
|---|---|
| Recommended PQ suite spelled out (L5, Q3, Q5) | [x] README |
| Classical-optimized suite (L1, L3) with quantum caveat | [x] README |
| Example CLI snippets defaulting PQ examples to L5 | [x] README |
| CI still exercises L1 (speed) plus cost-model assertions for Q floors | [x] existing CI |

Still open:

| Item | Status |
|---|---|
| Timed Q5 end-to-end run recorded in research reports | [ ] long wall-clock |
| Optional packaging: “pq” profile that only builds L5/Q3/Q5 | [ ] nice-to-have |

## 3. External review + kani4_sage + level-structure literature

| Item | Status |
|---|---|
| PR #1 open with REVIEW.md reproduction table | [x] |
| Reviewer checklist for Tier A vs Tier B claims | [x] docs/REVIEW.md |
| Open questions C3/C6/C7/C8 called out for reviewers | [x] SECURITY §8.1 |
| Rust `cryptanalysis::kani4` scaffolding + CI | [x] |
| Sage driver stub + setup instructions | [x] research/kani4_sage/ |
| Literature drill template (DFP / Arpin / Macula–Stange) | [x] docs/LITERATURE_DRILL.md |

Still open (needs external work or Sage):

| Item | Status |
|---|---|
| Run Theta_dim4 `KaniEndoHalf` on the published e_A=16 instance (eval correct) | [x] `research/kani4_sage/SMALLEST_RUN.md` |
| Recover an Eigen-Séta Toy16 isogeny image inside Theta_dim4 | [x] `attack_toy16.sage` (eval check true, 0.3 s) |
| Measure per-guess dim-4 cost; feed into security.rs oracle note | [ ] |
| Fill LITERATURE_DRILL.md pass/fail for each citation | [ ] cryptanalyst time |
| Peer review of docs/FO_PROOF.md | [ ] |
| Bound δ_corr / prove fixed-chain correctness | [ ] |
| Accept/merge cryptanalysis findings from PR #1 | [ ] |

## 4. Performance: field/asm and keygen parallelism

| Item | Status |
|---|---|
| Theta (2,2)-chain in KEM decaps | [x] |
| Parallel keygen attempts (`keygen_with_jobs`, CLI `--jobs`) | [x] |
| Re-tune attacker oracle costs after defender theta (document margin) | [~] note in SECURITY §5.1 |

Still open:

| Item | Status |
|---|---|
| NUMS primes: smallest c, fixed MR bases, `verify_nums` | [x] `docs/NUMS.md` |
| Montgomery-friendly mul for Q3/Q5 (raise limb cap 48 → 96) | [x] `fp::mul_friendly` |
| Hand-written assembly / Fiat-crypto codegen per prime | [ ] optional; friendly reduction is the main saving |
| Batch inversions / strategy tuning vs upstream Theta-Rust | [ ] |
| Parallelise inside a single keygen_attempt (not just multi-attempt) | [ ] |
| Fault-attack threat model | [ ] P2 |

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

Closing the research gaps in §3 “still open” is what moves the scheme from
“author cost model” toward “externally scrutinised”. That is the critical
path after this commit.
