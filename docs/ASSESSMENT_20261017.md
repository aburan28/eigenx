# Eigen-Séta v2: assessment of 2026-10-17

An authors'-side assessment of the scheme as implemented here, made for
revision 2 of the paper (`crypto:papers/eigen-seta/`, §"Assessment"). It is
not an external review. The full report, records, graphs and PDF are in
`crypto:research/eigen_seta_assessment_20261017/`.

## Verdict

No attack below the costs in `SECURITY.md` §3 was found. The binding attack
is still mask guessing (≈ 2^λ classical chains; 2^(λ/2) Grover queries on L1
and L3, ≥ 2^λ queries on L5, Q3 and Q5). Risks, in order:

1. **An effective orientation of E_A.** The public key is a length-2E segment
   of a horizontal 2-walk in the hidden Z[f·ω]-orientation (`SECURITY.md`
   §3.2 E). Making that orientation effective (level structure, sesquilinear
   pairings: `LITERATURE_DRILL.md` L6, L8) would give subexponential quantum
   key recovery through Kuperberg **at every set**: the Q sets enlarge d, not f.
2. **A mask-free dimension-4/8 embedding** (L2) would remove the √d mask search.
3. **The decisional assumption** (L7) has no argument beyond the absence of a
   distinguisher.

## Findings and what changed in this repository

| # | Finding | Class | Change |
|---|---|---|---|
| A1 | A candidate f can be tested on the public key alone: a (2^e,2^e) Kani chain on E_A × E_A for u·ψ_A + c, ψ_A = 2τ − t·f of degree 7f², with 2^e = 7u²f² + c² + c′². e ≈ 2·f_bits + 3 + O(1) against E ≈ 4·f_bits for decapsulation; no ciphertext needed. | attack-cost accounting, small | `cryptanalysis::pk_test`, `examples/pk_degree_test.rs`; `security.rs` charges the shorter chain (degree-search gate columns −1.9 bits; no floor or tier changes) |
| A2 | `Fp::mul2`/`mul3` (used by F_p² mul and square) took the Montgomery-friendly reduction for every modulus; `tests/field.rs::f1` (2^61 − 1) failed on every run on main. No parameter set affected (all friendly). | bug | guard added, as in `Fp::mul` |
| A3 | `SECURITY.md` §1 and §3.3 put the public ω-eigenbasis on E_D[3^F3]. Impossible: 3 is inert in Q(√−7); the cyclic-kernel proof needs no eigenline mod 3. | documentation | corrected |
| A4 | The cost model's admissible fraction 1/ln(2^E) is described as the density of sums of two squares. Key generation accepts f when 2^E − 7d²f², after trial division to 2^14, leaves 1 or a prime ≡ 1 (mod 4), plus the divisibility and sign conditions; the measured rate ρ (paper §6) exceeds the model's fraction by 1.6–2.3 bits. The model therefore understates the search, which is the conservative direction. | documentation | noted here; model unchanged |
| A5 | CI on main fails at `cargo fmt --check`, then at clippy `-D warnings` (rustc 1.97). | CI | `cargo fmt` and four clippy fixes, no behaviour change |
| A6 | Revision 1 of the paper conjectured NIST categories 1/3/5 for L1/L3/L5 without the tier qualification this repository already makes. | claim correction (paper) | paper revised to Tier A / Tier B |

Measured (run `run-20261017T1623Z-pk-degree-test`, records in the crypto
assessment directory): the true degree is accepted at e = 569 / 832 / 1087
at L1 / L3 / L5 (E = 1076 / 1594 / 2106). All 158 wrong candidates tested,
including near misses agreeing on 16, 64 and f_bits/2 low bits, are rejected
after a full-length chain. Median time per test against one decapsulation:
0.79 vs 1.28 s (L1), 2.72 vs 5.54 s (L3), 6.84 vs 13.34 s (L5), on a shared
4-vCPU VM.
