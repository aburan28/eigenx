# Literature drill: open quantum / structural questions

Author reading against the public key and ciphertext, 2026-10-13. This is
not an external review. Status: `no-attack` (the cited attack does not apply
as stated), `needs-experiment` (it might, and nobody has run it), or
`measured` (we ran the related experiment).

| ID | Question | Citation | Status | Note |
|---|---|---|---|---|
| L1 | Eigenbasis leakage | Wesolowski 2021/1583 | no-attack | Vectorisation and Kuperberg need an algorithm that evaluates the class-group action. The public key is two horizontal 2-walks in a hidden orientation. Without α that action is not computable (O-Uber / O-EndRing*). The cost model still lists f^{1/2} as a false-but-conservative classical lower bound (`SECURITY.md` §3.2 E). |
| L2 | Mask-free dim-4/8 | Robert 2022/1038; 2024/1180 | needs-experiment | **With** the honest masks, Theta_dim4 `KaniEndoHalf` on a Toy16 isogeny returns the Kani identity (`research/kani4_sage/TOY16_RUN.md`, 0.31 s). A mask-free embedding from the subgroups φ(⟨P⟩), φ(⟨Q⟩) alone is not implemented and is not ruled out. |
| L3 | Grover oracle / MAXDEPTH | NIST call; Jaques–Schanck | measured | `security::grover_gates_log2` is the parallel-Grover formula in `SECURITY.md` §5.1. The oracle is costed at 2^8 F_{p²} multiplications per (2,2)-step, below a theta implementation, so the defender’s theta chain does not reduce the attacker’s budget. A unit test checks parallel Grover is never cheaper than serial. External sign-off of the constant 2^8 is still open. |
| L4 | Degree-search pruning | — | no-attack | The only cheap filter in the code is “2^E − 7 d² f² is a sum of two squares with the 2-adic sign condition”, about 1/ln(2^E) of candidates (`SECURITY.md` §4.2). No tighter public predicate is implemented. |
| L5 | Composite f | SCALLOP / PEARL-SCALLOP | no-attack | Those attacks need an effective orientation to walk the class group of the conductor. Composite f does not hand that over. Restricting f to primes remains optional hardening, not applied (`SECURITY.md` §3.3). |
| L6 | Eigenbasis as level structure | De Feo–Fouotsa–Panny 2024/459; Arpin et al. 2024/1172 | needs-experiment | Both papers use level structure to make a group action effective. Mapping their public objects onto (E_A, eigenlines, pairing) has not been carried out. This row stays open for a cryptanalyst. |
| L7 | Decisional eigen-CIST | design note §5.2 | no-attack | Every basis is an eigenbasis of some endomorphism. The assumption is about the **size** of the smallest diagonal one (the secret degree), not existence. No distinguisher for that size is known here (`SECURITY.md` §3.3). |
| L8 | Sesquilinear pairings | Macula–Stange, as cited in `SECURITY.md` §8 | needs-experiment | Not instantiated against this public key. Left open on purpose. |
| L9 | Correctness error δ_corr | HHK; `FO_PROOF.md` | measured | Key generation rejects a key that fails one encaps/decaps self-test. `toy16_recovers_several_kernel_scalars` recovers s ∈ {0, 1, 2, 7, 1000, d/2, d−1}. That is evidence at Toy16, not a proof that every honest ciphertext on every set decapsulates. The FO bound keeps the `q_D · δ_corr` term. |

## Dim-4 experiment

The with-masks Toy16 run is `research/kani4_sage/TOY16_RUN.md`. Closing L2
as `no-attack` still requires a mask-free attempt, which this repository
does not contain.
