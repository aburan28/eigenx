# Eigen-Séta v2: security analysis

Revision 1, 2026-10-12, describing the implementation at commit `a204dbf`.
**Under review**: comments are collected on the pull request that carries
this document and as issues using the "Cryptanalysis finding" template;
`docs/REVIEW.md` explains how to reproduce every number here.

Status of this document: an internal analysis by the authors of the scheme,
with every attack that we know of costed, the two assumptions that the cost
model rests on tested experimentally at toy scale, and the literature that
each number depends on cited. **It is not a security proof and it has not
been reviewed by anyone outside the project.** Section 8 lists what is and
is not established.

The numbers in this document are produced by `cargo run --release --example
security_estimate` (cost model, `src/security.rs`) and `cargo run --release
--example cryptanalysis` (toy-scale attacks, `src/cryptanalysis/`), and
asserted in `tests/security.rs` and `tests/cryptanalysis.rs`.

## 0. Claims under review

Each claim is numbered so that a review comment can name it. "Evidence"
says what in this repository backs it; "kind" says whether that evidence is
a measurement, a calculation from stated assumptions, or a reading of the
literature.

| # | Claim | Evidence | Kind |
|---|---|---|---|
| C1 | The secret key is a deterministic function of the secret degree f and public data; it has no entropy beyond f. | `EigenSeta::secret_keys_from_degree`; `tests/cryptanalysis.rs::key_is_a_function_of_the_secret_degree`; §4.1 | measured (Toy16, every run) |
| C2 | A wrong degree f′ is rejected only at the final splitting step of the decapsulation chain, independently of how many low bits it shares with f; no bit-by-bit recovery of f. | `cryptanalysis::degree::near_miss_experiment`, `chain::inspect_chain`; §4.2 | measured (Toy16, k ≤ 40 of 50 bits) |
| C3 | Message recovery by mask guessing costs 2^e′ Kani evaluations with 2^(2e′) ≥ d classically and 2^(e′/2) under Grover, and is the binding attack on L1/L3/L5. | §3.1 B; `security::estimate` | calculation (FESTA's classical analysis + Grover); the Kani recovery step given correct masks is **not** implemented here |
| C4 | Generic meet-in-the-middle on the sender's 3^F3-isogeny costs d^{1/2}. | `cryptanalysis::mitm3`; §4.3 | measured (Toy16, 3/3 recoveries) |
| C5 | Key recovery by guessing f costs 2^(f_bits − 2)/ln(2^E) decapsulations; the sum-of-two-squares filter is the only cheap pruning. | §3.2 C; C1, C2 | calculation on top of C1, C2 |
| C6 | No sub-square-root quantum speed-up is available: the attacker holds no effective orientation, so Kuperberg does not apply; Tani is no better than vOW with memory costed. | §3.2 E, §5.2; Wesolowski 2021/1583, Jaques–Schanck | literature |
| C7 | Every known attack costs ≥ 2^λ classically on L1/L3/L5/Q3/Q5; L1/L3/L5 meet NIST categories 1/3/5 in the gate-count-under-MAXDEPTH metric; Q3/Q5 (and L5 as the level-1 set) cost ≥ 2^λ quantum queries. | `security::estimate`; `tests/security.rs`; §5 | calculation under the oracle costs of §5.1 |
| C8 | Self-pairing, lollipop and FESTA adaptive attacks do not apply as stated (disc = −7f² odd and coprime to 3; eigenlines belong to the secret α; constant-time decapsulation with implicit rejection). | §3.3; `cryptanalysis::checks`; `tests/robustness.rs`; `SIDE_CHANNELS.md` | literature + measured |
| C9 | The Q3/Q5 parameters are generated correctly and run end to end; there is no Q1 because the construction reproduces the L5 prime. | `examples/gen_params.rs` (regeneration is byte-identical); CLI run at Q3 | measured |

Questions we would most like reviewers to answer are in §8.1.

## 1. What the attacker sees

Public parameters: the prime p = 2^(E+1)·3^F3·c − 1, the CM(−7) curve E_D
(j = −3375) and a public basis of E_D[3^F3] consisting of eigenvectors of
its degree-2 endomorphism ω.

Public key: a curve E_A together with x(λ1·P), x(λ2·Q), x(P − Q), where
(P, Q) is the image under the secret degree-f isogeny φ_f : E_D → E_A of a
2^E-eigenbasis of ω, and λ1, λ2 are secret scalars.

Ciphertext: a curve E_C = E_A/⟨P_d + s·Q_d⟩ (3^F3-isogeny) and the images
of the public 2^E-basis under that isogeny, multiplied by masks (a, b) =
G(s, H(pk)); plus the FO ciphertext of the key.

Secret key: f, the Kani scalars (c, c′) with c² + c′² = 2^E − 7·d²·f² and a
2-adic sign condition, the scaled eigenvalues, and the public key. Section 3
shows that **everything in the secret key is a deterministic function of f
and public data** (`EigenSeta::secret_keys_from_degree`). The secret key
therefore carries exactly the entropy of f: about f_bits − 1 bits (f is odd,
uniform in [f_lo, 2 f_lo)).

## 2. Parameter sets

| Set | λ | p bits | E | F3 | log2 d | f bits | quantum-hardened |
|---|---|---|---|---|---|---|---|
| Toy16 | 16 | 216 | 174 | 21 | 33.3 | 50 | no |
| Toy32 | 32 | 369 | 297 | 41 | 65.0 | 80 | no |
| L1 | 128 | 1341 | 1076 | 162 | 256.8 | 277 | no |
| L3 | 192 | 1991 | 1594 | 243 | 385.1 | 409 | no |
| L5 | 256 | 2630 | 2106 | 324 | 513.5 | 536 | no |
| Q3 | 192 | 3904 | 3125 | 485 | 768.7 | 791 | yes |
| Q5 | 256 | 5192 | 4155 | 647 | 1025.5 | 1049 | yes |

"Quantum-hardened" means d = 3^F3 ≥ 2^(4λ) instead of 2^(2λ); see §5. There
is no Q1: at λ = 128, F3 = ⌈4·128 / log2 3⌉ = 324, which is exactly the L5
prime, so **L5 is the quantum-hardened level-1 set**.

## 3. Attacks and their cost

All costs are log2 of the number of operations and are deliberately in the
attacker's favour (memory ignored, oracles costed cheaply). "Classical" is
the best known classical algorithm; "q-queries" is the number of oracle
queries of the best known quantum algorithm; "gates" is the quantum gate
count under NIST's MAXDEPTH limits, computed as in §5.

### 3.1 Message recovery

**A. Meet-in-the-middle / vOW on the sender's isogeny.** E_A → E_C is a
3^F3-isogeny with both endpoints public. Generic MITM costs d^{1/2}
classically (vOW: time d^{3/4}/√memory, so d^{1/2} only with d^{1/2} memory)
and Tani's claw finding d^{1/3} quantum queries; Jaques–Schanck show that
with realistic memory costs Tani is no better than classical vOW.

**B. Mask guess + Kani ("eigen-CIST").** The Weil pairing of the masked
images leaks a·b·d mod 2^E, leaving ≈ 2^(E−1) masks. An attacker who guesses
the masks modulo 2^e′ with 2^(2e′) ≥ d can recover the isogeny by a Kani
computation in dimension 2 (the FESTA CIST analysis), so the cost is 2^e′ ≈
d^{1/2} Kani evaluations classically. **Grover halves the exponent**: 2^(e′/2)
Kani-oracle queries. FESTA's own analysis is classical only (it defers the
quantum case to "the quantum security analysis of SIDH"); the Grover count
here is ours and is the binding quantum attack on the L sets.

Why guessing modulo 2^e′ is enough: the sender's isogeny has degree d, and
Kani's lemma needs torsion of order N > d on E_A × E_C where N = 2^e′; the
images on the 2^e′-torsion are known once the masks are known modulo 2^e′.

### 3.2 Key recovery

**C. Secret degree search.** Because the secret key is a function of f and
public data, an attacker guesses f (2^(f_bits − 2) odd candidates), derives
the candidate key (`secret_keys_from_degree`; both sign choices work, §4.1),
and tests it with one decapsulation of a ciphertext whose key they know. Only
the ≈ 1/ln(2^E) candidates that admit Kani scalars need a decapsulation, so
the cost is 2^(f_bits − 2)/ln(2^E) decapsulations classically (2^265.5 at
L1); Grover gives the square root in decapsulation-oracle queries. f_bits ≈
2λ + 21, so this is far above A and B.

Two experimental facts underpin this row (§4): the derivation from the true
f does yield a working key (so the key has no entropy beyond f), and a wrong
f′ fails only at the last step of the chain regardless of how many low bits
it shares with f (so f cannot be recovered bit by bit).

**D. Endomorphism ring of E_A.** The generic problem: E_A is a random-looking
supersingular curve. Classical Õ(p^{1/2}) (Delfs–Galbraith; EHLMP) with a
2026 improvement to p^{1/3+o(1)} under a smoothness heuristic and with p^{1/3}
memory (Wesolowski 2026/1486; the o(1) hides a superpolynomial factor), and
p^{2/5+o(1)} unconditionally (Udovenko 2026/1575). Quantum Õ(p^{1/4})
(Biasse–Jao–Sankar). The cost model uses p/3 classically. Since p ≈ 2^(10.5λ)
this is never the binding attack.

Once a single non-scalar endomorphism α of E_A is known, End(E_A) follows in
|disc Z[α]|^{1/4} classically and L_{|disc|}(1/2) quantumly (Herlédan Le
Merdy–Wesolowski 2023/1448). Here disc Z[α] = −7f², so that would be ≈ 2^140
classically and subexponential quantumly at L1 — but α is the secret; the
attacker would have to recover f first (row C). This is why `decaps` is
constant time and FO uses implicit rejection: **leaking α through a side
channel reduces the scheme to a subexponential quantum problem.**

**E. Hidden orientation.** E_A carries the orientation by Z[f·ω]
(discriminant −7f²) pushed through φ_f. The public eigenlines ⟨P⟩, ⟨Q⟩ are
horizontal 2-isogeny walks of length E in that orientation, so they reveal
class-group information. In the class-group setting (Wesolowski 2021/1583)
the vectorisation problem costs |disc|^{1/4} = 7^{1/4}·f^{1/2} classically
and L(1/2) quantumly via Kuperberg — **but every one of those algorithms
requires evaluating the class-group action**, which here needs an effective
orientation on E_A, i.e. the secret. Without it the problem is Wesolowski's
O-Uber, equivalent to O-EndRing* (End(E_A) plus an orientation), for which
only exponential algorithms are known. The cost model lists f^{1/2} as a
lower bound under the false-but-conservative assumption that the action can
be evaluated; no quantum column is given because Kuperberg does not apply.

The self-pairing attacks (Castryck et al. CRYPTO 2023) need m² | disc for a
prime power m with accessible m-torsion, and an effective orientation on
both curves. disc = −7f² is odd and f is coprime to 21 (`checks.rs`), so
the 2- and 3-power torsion available over F_{p²} is excluded; any other m
would have to divide f, which is secret, and the orientation is hidden.

### 3.3 Not attacks, but checked

* **Distinguishing**: every basis of E_A[2^E] is an eigenbasis of *some*
  endomorphism, so what the public key hides is only the *size* of the
  smallest diagonal endomorphism. The scheme's IND-CPA assumption is thus
  decisional ("eigen-CIST", design note §5.2); this document does not
  establish it.
* **Lollipop attacks** (Castryck–Vercauteren 2023/1433): apply when a public
  basis point spans an eigenspace of Frobenius or of a *known* low-degree
  endomorphism. Here the eigenlines are of the secret α; E_D's ω eigenbasis
  on the 3^F3-torsion is public, but the sender's isogeny kernel ⟨P_d + s Q_d⟩
  is not an eigenline for s ≠ 0, ∞.
* **FESTA adaptive attacks** (Moriya–Onuki 2023/1092; MOXZ PQCrypto 2024;
  Zhou–Xu 2024/345) need a validation or timing oracle; decapsulation here
  is constant time with implicit rejection, and `tests/robustness.rs` checks
  that corrupted ciphertexts give the same rejection path as random ones.
  Mask-leakage (Das et al. PKC 2026) and fault attacks (Gilchrist–Lai–Meyer
  2026/1944) apply as to FESTA: masks are derived from the seed per
  ciphertext and never reused; fault resistance is out of scope.
* **Weak-key classes**: f is a uniformly random odd integer coprime to 21,
  not necessarily prime, so it has small factors with constant probability
  and is 2^64-smooth with probability ≈ ρ(277/64) ≈ 2^−9 at L1. Smoothness
  does not help any of rows A–E without an effective orientation (the
  attacker cannot enumerate degree-f isogenies without knowing f); SCALLOP
  and PEARL-SCALLOP choose a prime conductor for the *oriented* setting.
  Restricting f to primes would cost ≈ 200× more candidates in key
  generation and is listed as an optional hardening, not applied.

## 4. Experiments (toy scale)

Run: `cargo run --release --example cryptanalysis -- 3 4` at Toy16 (p 216
bits, E 174, F3 21, f 50 bits); the shorter versions are in
`tests/cryptanalysis.rs` and run in CI.

### 4.1 The key is a function of f

For every key tried (3/3 in the report run, plus every CI run),
`secret_keys_from_degree(pk, f)` with the true f produced a key that
decapsulated a fresh ciphertext to the right shared secret; in fact both
sign choices of the Kani scalars gave working keys (6/6 rows), so one
decapsulation per candidate f suffices. Per candidate f the derivation plus
one decapsulation costs 0.08–0.38 s at Toy16 (variable-time `two_squares`,
mostly). Consequence: the secret key has no entropy beyond f, and row C is
exactly 2^(f_bits − 2) decapsulations.

### 4.2 Wrong degrees fail only at the last step (near-miss localisation)

The design note (§5.1) claimed, unverified, that a wrong eigenvalue guess
makes the (2,2)-chain fail only at the end, so that no per-bit localisation
of the secret is possible. `near_miss_experiment` derives keys from wrong
degrees f′ ≡ f (mod 2^k) for k ∈ {0, 4, 8, 16, 24, 32, 40, 48} (of 50–51 bits)
and walks the decapsulation chain with the variable-time
`chain::inspect_chain`, which records whether the product kernel is
isotropic, whether gluing succeeds, the first step at which the Richelot
determinant vanishes (a premature split), and whether the scheduled final
split occurs.

Result of the report run (3 keys, 4 wrong degrees per level, 12 rows per
level; k = 48 leaves only three residue-class slots at 50 bits and none of
them admits Kani scalars, so that level is empty):

| rows | decaps ok | product kernel ok | glue ok | early split | final split | Richelot steps |
|---|---|---|---|---|---|---|
| true f (6) | 6 | 6 | 6 | 0 | 6 | 168 = E − 6 |
| k = 0, 4, 8, 16, 24, 32, 40 (12 each) | 0 | 12 | 12 | 0 | 0 | 168 |

Every derived key passes the product-kernel and gluing checks, no chain
splits early, none splits at the scheduled step, none decapsulates
correctly; the true f splits at the scheduled step and decapsulates. So the
per-step behaviour of the chain is identical for a wrong f′ agreeing with f
on 0 bits and on 40 of 50 bits: the chain carries no information about f
until the final splitting test. The claim stands at this scale; the
structural reason is that any f′ with consistent (c, c′) gives an isotropic
kernel of the right order, so the chain is a valid (2^E, 2^E)-isogeny of a
different abelian surface. (Only about 1/ln(2^E) of the candidate f′ admit
Kani scalars at all, exactly as in honest key generation; the attacker
skips the others with cheap arithmetic, so row C counts
2^(f_bits − 2)/ln(2^E) chain evaluations — about 2^9.5 fewer than the raw
candidate count at L1.)

### 4.3 Meet-in-the-middle on the sender's isogeny

`mitm3::attack` recovers the sender's scalar s from (pk, ct) at Toy16 by
enumerating 3^h kernels from E_A (h = ⌊F3/2⌋), hashing j-invariants, and
matching from E_C; the recovered s is confirmed by re-encryption. Cost shape:
3^h + 3^(F3−h) 3-isogeny chains, i.e. d^{1/2}. Report run, 3/3 keys
recovered: 3^10 = 78 732 chains from E_A, 79 000–222 000 from E_C before the
match (expected ≈ 3^11/2), 53 000–164 000 to extend s to the full 3^21, i.e.
2^18–2^19 chains and 45–70 s per key at 157 µs per chain. At L1 the same
attack is 2^128 chains.

### 4.4 Mask-guess + Robert/Kani (attack B scaffolding)

`cryptanalysis::kani4` finds Robert parameters `(e, a1, a2)` with
`2^e = d + a1² + a2²`, sets `e_mask = ⌈e/2⌉ + 2`, builds `form_kernel` on
`E_A² × E_C²`, and times one dim-2 `product_isogeny` of length
`min(e_mask, 12)` as the per-guess oracle cost. With the honest FO masks
(from a known `s`) it recovers `s` via the MITM stand-in; without known
masks it times guesses and does not claim recovery. Full dim-4 theta
evaluation is the Sage driver in `research/kani4_sage/` (Theta_dim4). At
Toy16, CI asserts Robert params exist, the honest-mask path confirms by
re-encryption, and a no-honest-mask run of 8 guesses claims nothing.

## 5. Quantum security

### 5.1 Method

NIST's categories are defined by the cost of key search on AES-128/192/256:
2^170, 2^233, 2^298 quantum gates **divided by MAXDEPTH**, for MAXDEPTH ∈
{2^40, 2^64, 2^96}, or 2^143, 2^207, 2^272 classical gates. For a Grover
search over 2^n items with an oracle of 2^g gates and depth 2^(g−4):

* serial Grover needs depth 2^(n/2 + g − 4); if that fits in MAXDEPTH the
  gate count is 2^(n/2 + g);
* otherwise 2^k machines run in parallel so each fits, and the total is
  2^(n + (g − 4) + g − MAXDEPTH) — Grover parallelises badly, which is what
  makes the depth limit bite.

Oracle costs: a (2,2)-chain of t steps at p bits is costed at t·2^8 F_{p²}
multiplications of 3·p_bits^{1.585} gates each (Karatsuba; below any known
theta-coordinate implementation), and a 3-isogeny step at 2^6 F_{p²}
multiplications. These are generous to the attacker.

### 5.2 Results (`security_estimate`)

| Set | classical floor | quantum queries | gates MD 2^40 / 2^64 / 2^96 | AES reference |
|---|---|---|---|---|
| L1 | 2^128.4 | 2^64.5 | 2^151 / 2^127 / 2^98 | 2^130 / 2^106 / 2^74 |
| L3 | 2^192.6 | 2^96.5 | 2^218 / 2^194 / 2^162 | 2^193 / 2^169 / 2^137 |
| L5 | 2^256.8 | 2^128.5 | 2^284 / 2^260 / 2^228 | 2^258 / 2^234 / 2^202 |
| Q3 | 2^384.4 | 2^192.5 | 2^415 / 2^391 / 2^359 | 2^193 / 2^169 / 2^137 |
| Q5 | 2^512.7 | 2^256.5 | 2^545 / 2^521 / 2^489 | 2^258 / 2^234 / 2^202 |

Reading:

* **L1/L3/L5 meet NIST categories 1/3/5 in NIST's gate-count metric at all
  three MAXDEPTH values** (the Grover oracle — a Kani chain of ≈ λ steps on a
  ≈ 10λ-bit field — is expensive, and the search does not parallelise
  within the depth limit). They do **not** meet the stricter "2^λ quantum
  queries" reading: the binding attack B needs 2^(λ/2) oracle queries, the
  same exposure as FESTA.
* **Q3/Q5 (and L5 as Q1) have ≥ 2^λ quantum queries and ≥ 2^(2λ) classical
  operations against every known attack**: d ≥ 2^(4λ) makes the mask space
  2^(2λ), so Grover's square root lands at 2^λ. The price is a prime about
  1.5× longer (3904 / 5192 bits) and correspondingly slower arithmetic.
* Quantum algorithms with more than a square-root speed-up are not known to
  apply: nothing the attacker can compute is a group action (the orientation
  is hidden, §3.2 E), so Kuperberg's sieve does not apply, and Tani's claw
  finding is not an improvement over classical vOW once memory is costed
  (Jaques–Schanck).

`tests/security.rs` asserts the classical floor ≥ λ for every set, the
quantum-query floor ≥ λ for the quantum-hardened sets, and the gate-count
floor ≥ the AES reference at every MAXDEPTH for L1/L3/L5.

## 6. Assumptions

1. **Eigen-CIST (decisional, message recovery).** Given (E_A, P, Q, E_C,
   [a]φP, [b]φQ) for random masks, the shared secret is indistinguishable
   from random. Best known attacks: A, B.
2. **Secret-degree EndRing (key recovery).** Given the public key, finding
   f (equivalently α, equivalently End(E_A) with its orientation) costs at
   least min(C, D, E). The eigenbasis hint is assumed to give nothing
   beyond the rows above; §4.2 is the only experimental evidence.
3. **FO transform.** IND-CCA in the ROM follows from the above via the
   HHK modular framework specialised to implicit rejection; the reduction
   is written in `docs/FO_PROOF.md`. It assumes decapsulation is correct
   for every honest ciphertext and constant time. Correctness has held in
   every run at every parameter set; a correctness proof for the fixed
   chain schedule is not written.

## 7. Implementation-level

* Decapsulation is constant time by construction (`SIDE_CHANNELS.md`):
  no secret-dependent branches, indices or early exits; field inversion and
  square roots use public exponents or Bernstein–Yang with a public step
  count. Key generation is variable time by design.
* Hostile inputs: `tests/robustness.rs` — random ciphertexts, non-canonical
  field encodings, cross-key ciphertexts, and every single bit-flip of an
  honest ciphertext are rejected into the implicit-rejection key, which
  depends only on (sk, ct); malformed public keys are rejected by
  encapsulation; corrupt secret-key files are rejected by `sk_from_bytes`.
* Known-answer tests pin the exact bytes at Toy16 (`tests/kat/`).

## 8. What is and is not established

Established, with evidence in this repository:

* The secret key is a deterministic function of f and public data (§4.1).
* A wrong f′ is indistinguishable from any other wrong f′ at every step of
  the decapsulation chain except the last, at toy scale (§4.2).
* A generic MITM attack on the sender's isogeny works as costed (§4.3).
* Mask-guess + Robert/Kani scaffolding at Toy16: Robert parameters,
  `form_kernel`, product-isogeny oracle cost, honest-mask recovery
  (`cryptanalysis::kani4`; full dim-4 theta eval in `research/kani4_sage/`).
* Costs of all known attacks under the stated models (§3, §5).
* Correct, constant-time decapsulation (Mumford schedule; theta chain
  available and cross-checked on synthetic kernels); hostile-input
  behaviour (§7).
* Author-written IND-CCA reduction for the FO transform (`docs/FO_PROOF.md`).

Not established:

* Hardness of eigen-CIST and of secret-degree EndRing: no reduction to a
  standard problem. The decisional assumption in particular has no
  argument beyond "every known attack costs what §3 says".
* Any statement about attacks not yet invented, in particular attacks that
  use the public eigenlines as level structure (De Feo–Fouotsa–Panny
  2024/459; Arpin et al. 2024/1172) or sesquilinear pairings
  (Macula–Stange). Neither has been worked out for this scheme.
* A dimension-4 or dimension-8 variant of attack B that avoids guessing the
  masks: Robert's dimension-8 attack needs the images of a known-order
  torsion basis, which the masks hide; whether the mask-independent
  *subgroups* φ(⟨P⟩), φ(⟨Q⟩) suffice for a higher-dimensional embedding is
  open. The *with-masks* dim-4 path is scaffolded in Rust and stubbed for
  Theta_dim4 in Sage; it does not remove the mask guess.
* Peer review of `docs/FO_PROOF.md` (author-written only).
* External review. Nobody outside the project has looked at this.

### 8.1 Questions for reviewers

1. **Eigenbasis leakage (C5, C6).** The public key exposes two horizontal
   2-isogeny walks of length E in the hidden Z[f·ω]-orientation of E_A. We
   treat this as useless without an effective orientation. Is there a way
   to use the two walks together (they are eigenlines of the *same*
   endomorphism, with the pairing of their scaled generators public) that
   does not require evaluating the class-group action?
2. **Mask-free higher-dimensional embedding (C3).** The subgroups
   φ(⟨P⟩), φ(⟨Q⟩) ⊂ E_C are mask-independent. Can a dimension-4 or -8 Kani
   embedding be built from the subgroups alone, or from the subgroups plus
   the pairing value a·b·d mod 2^E, without guessing the masks?
3. **Oracle costing (C7).** The Grover oracle is costed at 2^8 F_{p²}
   multiplications per (2,2)-step with Karatsuba field multiplication and
   depth = gates/16. Is this defensible as a lower bound given
   theta-coordinate implementations, and does the NIST parallel-Grover
   accounting apply as used in `security::grover_gates_log2`?
4. **Degree-search pruning (C5).** Only f′ with 2^E − 7d²f′² a sum of two
   squares (with a 2-adic sign condition) yield a key. Is there any cheaper
   test than the sum-of-two-squares check that prunes further, or any
   structure in the set of admissible f′ that an attacker could enumerate
   faster than by sampling?
5. **Composite f (C8).** f is a random odd integer coprime to 21, not a
   prime. SCALLOP-type attacks on smooth conductors need an effective
   orientation. Is there a weak-key class for composite or smooth f that
   does not?
6. **Level-structure attacks (§8).** Do the reductions of De Feo–Fouotsa–
   Panny (2024/459) or the level-structure group action of Arpin et al.
   (2024/1172) give anything with the eigenbasis as level structure?
7. **Decisional assumption.** The IND-CPA assumption is decisional
   eigen-CIST (§6, assumption 1). Is the problem even well posed given that every basis
   is an eigenbasis of *some* endomorphism (§3.3), and is there a
   distinguisher based on the size of the smallest diagonal endomorphism?
8. **Correctness and FO (§6, assumption 3).** Decapsulation has never failed on an
   honest ciphertext, but there is no proof that the fixed chain schedule
   is correct for every key. A correctness failure rate ε would enter the
   FO bound; what is the right way to bound it?

## 9. Sources

* A. Basso, L. Maino, G. Pope, *FESTA: Fast Encryption from Supersingular
  Torsion Attacks*, ASIACRYPT 2023, ePrint 2023/660.
* W. Castryck, T. Decru, *An efficient key recovery attack on SIDH*,
  EUROCRYPT 2023, ePrint 2022/975. L. Maino, C. Martindale, L. Panny,
  G. Pope, B. Wesolowski, *A direct key recovery attack on SIDH*,
  EUROCRYPT 2023, ePrint 2023/640. D. Robert, *Breaking SIDH in polynomial
  time*, EUROCRYPT 2023, ePrint 2022/1038.
* W. Castryck, M. Houben, S.-P. Merz, M. Mula, S. van Buuren,
  F. Vercauteren, *Weak instances of class group action based cryptography
  via self-pairings*, CRYPTO 2023, ePrint 2023/549.
* W. Castryck, F. Vercauteren, *A polynomial time attack on instances of
  M-SIDH and FESTA*, ASIACRYPT 2023, ePrint 2023/1433.
* A. Herlédan Le Merdy, B. Wesolowski, *The supersingular endomorphism ring
  problem given one endomorphism*, IACR CiC 2(1) 2025, ePrint 2023/1448.
* B. Wesolowski, *Orientations and the supersingular endomorphism ring
  problem*, EUROCRYPT 2022, ePrint 2021/1583.
* C. Delfs, S. D. Galbraith, *Computing isogenies between supersingular
  elliptic curves over F_p*, DCC 78 (2016), arXiv:1310.7789.
  K. Eisenträger, S. Hallgren, K. Lauter, T. Morrison, C. Petit,
  *Supersingular isogeny graphs and endomorphism rings*, EUROCRYPT 2018.
* B. Wesolowski, *The supersingular isogeny problem in time and memory
  p^{1/3+o(1)}*, ePrint 2026/1486. A. Udovenko, *Solving the supersingular
  isogeny problem in time p^{2/5+o(1)}*, ePrint 2026/1575.
* J.-F. Biasse, D. Jao, A. Sankar, *A quantum algorithm for computing
  isogenies between supersingular elliptic curves*, INDOCRYPT 2014.
* S. Jaques, J. M. Schanck, *Quantum cryptanalysis in the RAM model:
  claw-finding attacks on SIKE*, CRYPTO 2019, ePrint 2019/103.
* NIST, *Submission Requirements and Evaluation Criteria for the
  Post-Quantum Cryptography Standardization Process*, Dec. 2016, §4.A.5.
* L. De Feo, T. B. Fouotsa, P. Kutas, A. Leroux, S.-P. Merz, L. Panny,
  B. Wesolowski, *SCALLOP*, PKC 2023, ePrint 2023/058. B. Allombert et al.,
  *PEARL-SCALLOP*, PKC 2025, ePrint 2024/1744.
* T. Moriya, H. Onuki, ePrint 2023/1092; T. Moriya, H. Onuki, M. Xu,
  G. Zhou, *Adaptive attacks against FESTA*, PQCrypto 2024; G. Zhou, M. Xu,
  ePrint 2024/345; S. Das, R. Invernizzi, P. Kutas, J. Meers, *Leveled
  isogeny problems with hints*, PKC 2026, ePrint 2025/2075; V. Gilchrist,
  Y.-F. Lai, M. Meyer, *Fault injection attacks on torsion masking*, ePrint
  2026/1944.
