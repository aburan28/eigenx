# Reviewing the Eigen-Séta security analysis

This page is for people asked to review `docs/SECURITY.md` ("the paper"),
the parameter choices, or the cryptanalysis code. It says what is being
claimed, how to reproduce every number, and where to put comments.

## What to review

The paper's §0 numbers the claims (C1–C9) and §8.1 lists the questions we
most want answered. Review can be at any of three depths:

1. **Paper only.** Read `docs/SECURITY.md`. Comment inline on the pull
   request that carries it, naming the claim (e.g. "C3") where relevant.
2. **Paper + cost model.** Also read `src/security.rs` (≈250 lines: the
   attack list, oracle costs, Grover-under-MAXDEPTH accounting) and
   `src/sets.rs` (the parameter table the model reads from).
3. **Paper + experiments.** Also read `src/cryptanalysis/` (`degree.rs`:
   key from degree and the near-miss experiment; `mitm3.rs`: the
   meet-in-the-middle; `kani4.rs`: mask-guess + Robert/Kani scaffolding;
   `checks.rs`: structural key checks), `src/theta.rs` / 
   `chain::decaps_chain_theta`, and `src/chain.rs::inspect_chain` (the
   per-step chain diagnostics the near-miss experiment relies on). The
   scheme itself is `src/kem.rs`; its design note is
   `docs/RESEARCH_ENDRING_PKE_STRATEGY.md`; the FO reduction is
   `docs/FO_PROOF.md`.

The question that matters most is not whether the numbers are computed
correctly from the model but whether the model is missing an attack. §8 of
the paper lists what we know we have not examined.

## Reproducing the numbers

Requirements: Rust 1.93 or later (CI uses 1.99). Builds are slow on
network or external volumes; set `CARGO_TARGET_DIR=/tmp/eigenx-target` if
that applies. All timings below are one core of an Apple laptop.

| What | Command | Expect | Time |
|---|---|---|---|
| Whole test suite (KEM, KATs, hostile inputs, CLI, toy attacks, cost-model assertions) | `cargo test --release` | all green; `tests/security.rs` prints the attack table and the MAXDEPTH comparison | ≈5 min + build |
| Attack cost table (paper §3, §5.2) | `cargo run --release --example security_estimate` | the table in §5.2, to one decimal | <1 s |
| Toy-scale attack report (paper §4) | `cargo run --release --example cryptanalysis -- 3 4` | `structural checks 3/3, key-from-degree 3/3, MITM 3/3, Kani4 3/3`; near-miss table with `decaps_ok`, `early_split`, `final_split` all 0 for every k and 6/6 for the true f | ≈4–6 min |
| FO IND-CCA writeup | read `docs/FO_PROOF.md` | HHK reduction under OW-CPA + key secrecy + δ_corr | — |
| Parameter regeneration (paper §2, C9) | `cargo run --release --example gen_params > /tmp/p.rs && diff /tmp/p.rs src/params_generated.rs` | no output from `diff` | ≈10 min |
| L1 round trip from the CLI | see the README's first code block | shared secrets and plaintext match | keygen 30 s, decaps 8 s |
| Q3 round trip from the CLI | same with `--set Q3` | same | keygen 25 min, decaps 9 min |
| Constant-time check of decapsulation | `SIDE_CHANNELS.md` describes the instruction-count method | counts independent of the secret | see that file |

The report is deterministic: all randomness in the examples and tests comes
from `drbg::Drbg::for_vector(label, index)` (SHAKE256), so two machines
produce the same keys, the same wrong degrees and the same MITM counts.

To change the model and see the effect, edit `src/security.rs::estimate`
and rerun `security_estimate`; `tests/security.rs` will tell you if a
parameter set drops below its level.

## Where to put comments

* **Line-level comments on the paper or code:** on the review pull request.
* **A concrete attack, a flaw in the cost model, or a parameter you think
  is wrong:** open an issue with the "Cryptanalysis finding" template. It
  asks for the claim number, the attack sketch or calculation, and the
  cost you get, so that it can be turned into a row of `security::estimate`
  or an experiment in `src/cryptanalysis/`.
* **Anything that should not be public before it is fixed** (a practical
  break at the L sizes): email the maintainer instead of opening an issue.
  This is research code that nobody should be using to protect data, but
  we would still rather fix the document before the finding circulates.

Comments that say "this is unconvincing" without a specific reason are
still useful; please leave them.

## What a finding looks like once accepted

A finding that changes a cost becomes a row (or a changed formula) in
`src/security.rs`, with the source cited in the note string and in
`docs/SECURITY.md` §9. A finding that can be run at toy scale becomes a
function in `src/cryptanalysis/` and a test. A finding that breaks a
parameter set makes `tests/security.rs` fail until the set is regenerated
or withdrawn. The paper's revision number and commit hash at the top are
bumped with each accepted finding.
