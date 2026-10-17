# Eigen-Séta specification (draft)

**Not ready to upload.** This text specifies the KEM implemented in
`src/kem.rs` at the parameter sets in `src/params_generated.rs`. If this
text and the code disagree, the code is the artifact under test and this
text is wrong until it is corrected. It does not invent a security claim
beyond `docs/SECURITY.md`.

**A submission that recommends L1 or L3 as NIST Category 1 or 3 is a false
submission.**

## 0. Map to the NIST PQC call

The call asks for a cover sheet, a complete written specification, a design
rationale, a performance analysis, known-answer tests, an expected-strength
statement, an analysis of known attacks, advantages and limitations, and an
intellectual-property statement, plus reference and optimized
implementations.

| Call item | Where | In this draft |
|---|---|---|
| Cover sheet | [`COVER.md`](COVER.md) | Placeholder |
| Written specification | §§1–4 | Draft |
| Parameter sets and sizes | §5 | From the implementation |
| Expected strength | §6 | Tier A / Tier B, as in `docs/SECURITY.md` |
| Known attacks | §6, and `docs/SECURITY.md` | Cost model, not a proof |
| Known-answer tests | §7 | Short L1 set |
| Design rationale | §8 | Draft |
| Advantages and limitations | §9 | Draft |
| Performance | §10 | Cited measurements only |
| Intellectual property | §11 | Placeholder, no patent claim |
| Reference / optimized C | [`GAPS.md`](GAPS.md) | Absent |

## 1. Data and parameters

The scheme is a KEM. Encapsulation and decapsulation are deterministic
functions of an explicit seed and of the secret key. The Fujisaki–Okamoto
transform with implicit rejection is part of the KEM, not an external
wrapper.

### 1.1 Field

Each parameter set fixes a prime

```
p = 2^(E+1) · 3^F3 · c − 1
```

with `p ≡ 3, 5, or 6 (mod 7)`. The cofactor `c` is the smallest positive
integer for which this `p` passes the fixed-base Miller–Rabin test in
`ntheory::is_nums_prime`. The recipe is `docs/NUMS.md`. This draft does not
change `c` for L1, L3, L5, Q3, or Q5.

`d = 3^F3`. Arithmetic is in `F_{p²}`, represented as `c0 + c1·i` with
`i² = −1` (`src/fp2.rs`). The public curve model used on the wire is the
Montgomery curve `y² = x³ + A x² + x`.

### 1.2 Byte encoding

An element of `F_p` is the big-endian canonical integer on `ceil(log2(p)/8)`
bytes, rejected if the integer is ≥ p (`Fp::from_bytes_be`). An element of
`F_{p²}` is `c0 || c1`. A public key or a KEM ciphertext is four field
elements, concatenated (`fbytes` in `src/kem.rs`):

```
pk = A || x(λ1·P) || x(λ2·Q) || x(P − Q)
ct = A_C || x(S) || x(T) || x(S − T)
```

`S` and `T` are the masked images defined in §3. Lengths are in §4. There
is no 8-byte `EIGX` header in these objects. The `eigenx` file format
(`src/wire.rs`) is a container around them and is not part of the KAT.

The shared secret is 32 bytes for every set.

### 1.3 Hash

`SHAKE256` is used as the extendable output (`sha3` crate). Domain strings
are the raw prefix bytes below, concatenated with the listed arguments.
This is the instantiation of the random oracles in `docs/FO_PROOF.md`.

| Oracle | Prefix | Arguments |
|---|---|---|
| `P` | `EigenSeta-P` | public-key bytes |
| `G` | `EigenSeta-G` | kernel-scalar bytes, then `P(pk)` |
| `K` | `EigenSeta-K` | kernel-scalar bytes, then ciphertext bytes |
| `R` | `EigenSeta-R` | 32-byte rejection seed `z`, then ciphertext bytes |

`P(pk)` is 32 bytes. `K` and `R` output 32 bytes. `G` outputs
`2 · ceil(E/8)` bytes, which are reduced modulo `2^E` and then forced odd
by setting the least significant bit of each half (`masks` in `src/kem.rs`).

The KAT DRBG is a different domain, `EigenSeta-DRBG`. See §6.

## 2. Key generation

`EigenSeta::keygen` (`keygen_attempt`). Variable time. It runs in a trusted
environment (`SIDE_CHANNELS.md`). The secret key does not store the degree
`f`. Decapsulation uses the derived scalars.

`E_D` is the curve of `j = −3375` over `F_p`, with the checked-in
coefficients `ED_A4_HEX`, `ED_A6_HEX`. `ω` is its degree-2 endomorphism
(`Omega::find`).

Repeat an attempt until one returns a key.

1. Draw `f` uniformly from `[f_lo, 2 f_lo)` (`F_LO_HEX` in the parameter
   set) and force it odd. Reject unless `kani_scalars_for_degree(f, flip =
   false)` returns Kani scalars `(c, c′)`. That function rejects `f` even or
   divisible by 3 or 7, rejects `2f ≡ 2^E (mod d)`, rejects
   `7 d² f² ≥ 2^E`, and requires `2^E − 7 d² f²` to be a sum of two squares
   with a sign choice whose 2-adic valuation matches the fixed schedule
   (`v2(c′ ∓ λ_s) = 2` for a choice of signs).
2. Search for an endomorphism `α` of `E_D` of norm `f · (2^E − f)` by the
   Cornacchia search in `keygen_attempt` (at most 20000 trials for the
   norm equation; a failed search retries the whole attempt).
3. Evaluate the Kani kernel `{([f]R, α(R))}` as a `(2^E, 2^E)`-isogeny on
   `E_D × E_D`. If `v2(p+1) ≥ E+2` the chain is
   `keygen_chain_theta`; otherwise it is `keygen_chain`. Among the
   checked-in sets, L1 (`c = 144`) and L5 (`c = 380`) take the theta path.
   L3, Q3, and Q5 have odd `c`, so `v2(p+1) = E+1` and key generation uses
   the Mumford chain. Decapsulation always uses the theta chain (§3).
   The kernel is generated by the images of a uniformly random basis
   `(R1, R2)` of `E_D[2^level]` (`level = E+2` on the theta path, else `E`).
   Each attempt draws it, and the 3^F3 basis of step 4, as `(aP+bQ, cP+dQ)`
   with `a, b, c, d` uniform modulo `ℓ^k` and `ℓ ∤ ad − bc`
   (`rerandomize_basis`), from a basis `(P, Q)` sampled once per key
   generation (`torsion_basis_cof`). This is the same distribution as a
   fresh basis per attempt, without the cofactor multiplications.
4. A short Weil pairing of order `3^min(8, F3)` selects the factor of
   degree `f`. The eigenbasis of that factor is recovered, moved to a
   Montgomery model of the right twist, and rescaled by secret odd scalars
   `λ1, λ2` modulo `2^E`.
5. The public key is the four-element encoding in §1.2. The secret key is
   `secret_key_from_parts`: `λ_s, λ_t, c′` modulo `2^E`, `c, c′` modulo `d`,
   a fresh 32-byte `z`, the bit length of `f`, and a copy of the public key.
   `f` itself is discarded.
6. **Self-test.** The attempt encapsulates once more under the new public
   key and decapsulates. If the shared secrets differ, the attempt is
   rejected. This draw is part of the KAT `keygen_coins` (§6). It is one
   honest ciphertext, not a proof that every ciphertext decrypts.

Secret-key layout (`sk_to_bytes`), little-endian scalars, no header:

```
λ_s || λ_t || c′_2 || c_3 || c′_3 || z || f_bits || pk
```

`λ_s`, `λ_t`, and `c′_2` each occupy `ceil(E/8)` bytes. `c_3` and `c′_3`
each occupy `ceil(d_bits/8)` bytes, where `d_bits` is the bit length of
`d`. `z` is 32 bytes. `f_bits` is 8 bytes, little-endian. `pk` is the
public key. `sk_from_bytes` rejects a wrong length or a public key that
fails `pk_points_valid`, and reduces the scalars into range.

`EigenSeta::secret_keys_from_degree` rebuilds candidate secret keys from
`f` and the public key. The KEM does not need it. It is why the secret key
has no entropy beyond `f` (`docs/SECURITY.md`, claim C1).

## 3. Encapsulation

`EigenSeta::encaps`. Returns nothing if the public key is the wrong length,
an element is not canonical, the Montgomery curve is singular (`A² = 4`),
the differential point is inconsistent with `x(P) − x(Q)`, or `{P, Q}` is
not an independent basis of the `2^E`-torsion (`pk_points_valid`).

Otherwise:

1. `pkh ← P(pk)`.
2. Draw
   `seed_bytes = ceil(d_bits/8) + ceil(λ/8)`
   uniform bytes. Interpret them as a little-endian integer and reduce
   modulo `d`. Call the residue `s`. The extra `ceil(λ/8)` bytes are there
   so that the statistical distance of the reduction is at most about
   `2^{−λ}`. L1 draws 49 bytes (33 + 16).
3. Let `(P_d, Q_d)` be the canonical basis of `E_A[d]` from
   `canonical_basis_3` (deterministic, public, variable time, bounded
   search). The kernel point is `P_d + s Q_d`.
4. Walk the `3^{F3}`-isogeny (`chain3`). Push the public key's three
   x-coordinates through it. Let `E_C` be the codomain and `φ` the isogeny.
5. `(a, b) ← G(s, pkh)`, odd modulo `2^E`. Set
   `S = [a]φ(P)`, `T = [b]φ(Q)`, and the third coordinate to
   `x([a](φ(P) − [b a^{−1}] φ(Q)))`, which is `x(S − T)` for that choice
   of signs.
6. `ct` is the four-element encoding of `(A_C, x(S), x(T), x(S − T))`.
7. The shared secret is `K(s, ct)`.

`enc_det` / `enc_det_core` are the deterministic map `s ↦ ct`. Encapsulation
is that map plus the seed and `K`.

## 4. Decapsulation

`EigenSeta::decaps`. Two phases.

**Public prepare** (`decaps_prepare`), variable time, public inputs only.
Parse `ct`. Reject a singular `A_C` or an x-coordinate that is not on the
curve. Lift `S` and `T` and flip the sign of `T` if needed so that
`x(S − T)` matches the ciphertext. Compute canonical `3^{F3}` bases of
`E_C` and of `E_A`. If this fails, return `R(z, ct)` immediately.

**Secret core** (`decaps_core`). Intended to be constant time in the secret
key and in `s`. Build the Kani kernel from `(c′, λ_s, λ_t)` acting on
`S` and `T`, evaluate `decaps_chain_theta`, and recover `s` by the pairing
/ discrete-log steps in `recover_s_core`. Re-encrypt with `enc_det_core`.
If the re-encrypted ciphertext equals `ct` as bytes, return `K(s, ct)`.
Otherwise return `R(z, ct)`. The comparison and the selection are
constant-time (`ct::bytes_eq`, `conditional_select`).

The core does not print a timing. The `eigenx decaps` command does not
print a timing. The KAT generator does not print a timing.

## 5. Parameter sets and sizes

Shared secret: 32 bytes on every set. Public-key and ciphertext lengths are
`4 · byte_len(F_{p²})`. Secret-key length is the layout in §2. Figures
match `set_info` / the README table.

| Set | λ in the parameter | p bits | E | F3 | log2(d) | f bits | pk = ct | sk | encaps seed |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| L1 | 128 | 1341 | 1076 | 162 | 256.8 | 277 | 1344 | 1855 | 49 |
| L3 | 192 | 1991 | 1594 | 243 | 385.1 | 409 | 1992 | 2730 | 73 |
| L5 | 256 | 2630 | 2106 | 324 | 513.5 | 536 | 2632 | 3594 | 97 |
| Q3 | 192 | 3904 | 3125 | 485 | 768.7 | 791 | 3904 | 5311 | 121 |
| Q5 | 256 | 5192 | 4155 | 647 | 1025.5 | 1049 | 5192 | 7050 | 161 |

`λ` in this table is `ParamSet::LAMBDA`. It is the classical sizing
parameter. It is not, by itself, a NIST category. There is no Q1 row: at
128 bits the quantum-hardened degree condition produces the L5 prime.
`SetId::from_name("Q1")` is an alias for L5.

Toy16 and Toy32 are not in this table and are not generated.

## 6. Security claims

Read this section before using any row above as a category.

### 6.1 The false-submission line

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
not a quantum category 5 set.

Q3 is claimed as category 3: ≥ 2^192 quantum queries against every known
attack in the cost model. Q5 is claimed as category 5: ≥ 2^256 quantum
queries against every known attack in the cost model.

### 6.2 What “claimed” means

The query and gate figures are the output of `security::estimate`
(`src/security.rs`), the same numbers tabulated in `docs/SECURITY.md` §5.2
and checked by `tests/security.rs`. Classical floor is the minimum over the
attacks that file implements. Quantum-query floor is the minimum over the
attacks that have a quantum-query column. Gate floors use the MAXDEPTH
parallel-Grover accounting in `grover_gates_log2`, with an oracle the model
deliberately prices cheaply for the attacker.

| Set | Classical floor | Quantum queries | Gates at MAXDEPTH 2^40 / 2^64 / 2^96 | What this draft submits |
|---|---|---|---|---|
| L1 | 2^128.4 | 2^64.5 | 2^151 / 2^127 / 2^98 | Tier A gate table only. Not Category 1. |
| L3 | 2^192.6 | 2^96.5 | 2^218 / 2^194 / 2^162 | Tier A gate table only. Not Category 3. |
| L5 | 2^256.8 | 2^128.5 | 2^284 / 2^260 / 2^228 | Quantum category 1 and classical category 5. |
| Q3 | 2^384.4 | 2^192.5 | 2^415 / 2^391 / 2^359 | Category 3. |
| Q5 | 2^512.7 | 2^256.5 | 2^545 / 2^521 / 2^489 | Category 5. |

The L1 and L3 gate columns sit above the AES-128 / AES-192 Grover
references used in `aes_grover_gates_log2` (2^170, 2^233, 2^298, divided by
MAXDEPTH). That comparison is the Tier A argument. It is not a category
assignment. The binding quantum attack on L1 and L3 is Grover search over
the mask guess, about 2^{λ/2} oracle queries (2^64.5 and 2^96.5 in the
model). NIST categories are the resources required to break AES. This
package does not treat a gate-count margin on a 2^{λ/2}-query attack as
Category 1 or Category 3.

L5's query floor is 2^128.5, which meets a 2^128 query bar and does not meet
a 2^256 query bar. Its classical floor is 2^256.8. That is why the claim is
quantum category 1 and classical category 5, not quantum category 5.

Q3 and Q5 take `d ≥ 2^{4λ}`, so the same Grover step lands at ≥ 2^λ
queries. The model reports 2^192.5 and 2^256.5.

### 6.3 Assumptions, not theorems

- **Eigen-CIST** (message recovery) and **secret-degree EndRing** (key
  recovery) are assumptions. `docs/SECURITY.md` §6 states them.
  `docs/SECURITY.md` §8 says they have no reduction to a standard problem.
- **FO.** `docs/FO_PROOF.md` is an author-written ROM argument, checked
  against the domain strings, odd masks, re-encryption, and implicit
  rejection in `src/kem.rs`. It has not been externally reviewed. The bound
  keeps a `q_D · δ_corr` term. `δ_corr = 0` is not proved.
- **QROM IND-CCA is not claimed** for any set, including L5, Q3, and Q5.
  `docs/FO_PROOF.md` §4 says the reduction is ROM, not QROM.
- No claim is made about attacks that are not in the cost model. The open
  questions in `docs/SECURITY.md` §8.1, including a mask-free
  higher-dimensional embedding, are open. [`GAPS.md`](GAPS.md) repeats the
  ones that block an upload.

## 7. Known-answer tests

### 7.1 Command

From the repository root, after a release build:

```
eigenx kat --set L1 --count 2 --out docs/nist/kat
```

The binary is `src/bin/eigenx.rs`. It calls `kat_nist::write_kem_kat`.
The command writes three files:

```
docs/nist/kat/PQCkemKAT_L1_1855.req
docs/nist/kat/PQCkemKAT_L1_1855.int
docs/nist/kat/PQCkemKAT_L1_1855.rsp
```

The `1855` is `sk_bytes` for L1. Each record has a `count` line.

To extend the L1 set, rerun with a larger `--count` (up to 10000). Count
100 is the length used by the NIST example generators. This draft checks
in count 2 because one L1 key generation includes a decapsulation self-test
and is too slow to treat a count of 100 as part of producing the draft.
The same command accepts `L3`, `L5`, `Q3`, `Q5`, and `Q1` (an alias of L5).
Those files are not checked in. Do not pass a toy prime; the CLI has no toy
set.

### 7.2 DRBG

Not NIST's AES-256-CTR `randombytes`. A file produced by that DRBG will not
match.

`Drbg` (`src/drbg.rs`) is SHAKE256 of the domain `EigenSeta-DRBG`, the seed
length as a little-endian `u64`, and the seed. The master DRBG is
`Drbg::new("EigenSeta-NIST-KAT-v1-{set}")`. Vector `i` takes the next 48
bytes of that stream as `seed`. `keygen` then `encaps` read one DRBG
initialised with that seed, in that order.

`keygen_coins` in the `.int` file is every byte `keygen` read, including
the encapsulation seed drawn by the self-test in §2. `encaps_coins` is the
later draw, and its length must equal the encaps seed length in §3 and §5
(49 bytes on L1).
The generator refuses to write the files if that length differs or if
decapsulation of the recorded ciphertext does not return the recorded
shared secret. It does not print a decapsulation time.

### 7.3 File format

Request (`.req`): `count`, `seed` (uppercase hex), then empty `pk`, `sk`,
`ct`, `ss`.

Intermediate (`.int`): `count`, `seed`, `keygen_coins`, `encaps_coins`.

Response (`.rsp`): `count`, `seed`, `pk`, `sk`, `ct`, `ss`, uppercase hex,
raw algorithm bytes (no `EIGX` header).

Comment lines state the set, the lengths, the DRBG, and the claim sentence
for that set. The L1 header contains the false-submission sentence.
`tests/kat.rs::nist_kat_l1_rsp_decaps_matches` checks that the checked-in
response decapsulates and that the header still contains that sentence.

The separate one-vector file `tests/kat/l1.txt` uses two labelled DRBG
streams (`L1-keygen` and `L1-encaps`). It is not the NIST-style file.

## 8. Design rationale

Eigen-Séta v2 makes the degree `f` of the trapdoor endomorphism secret.
The earlier public-degree `walk` variant is broken: its decryption key is a
function of public data (repository README, §4). The public key carries a
scaled eigenbasis of the secret endomorphism on `E_A[2^E]`, not the degree.

Encapsulation hides a `d`-isogeny by masking the torsion images with
`(a, b) = G(s, P(pk))`. The classical cost of guessing those masks, then
running Kani, is about `d^{1/2}`. Grover reduces the exponent by half. The
L sets take `d ≥ 2^{2λ}`, so the classical floor meets `λ` and the quantum
query floor is about `λ/2`. The Q sets, and L5 in its role as the
quantum-hardened 128-bit set, take `d ≥ 2^{4λ}` so the query floor meets
`λ`. That is the whole reason L1 and L3 are not submitted as categories.

The FO transform uses implicit rejection (`R(z, ct)`) so a decapsulation
failure is not a separate observable key, and re-encryption so a mangled
ciphertext does not return `K` of a recovered scalar. Masks are odd so that
distinct mask pairs are less likely to collide modulo `2^E`.

Primes are nothing-up-my-sleeve, as in §1.1. Decapsulation uses a fixed
public theta-chain schedule so the step count does not depend on the
secret. Key generation stays variable time.

## 9. Advantages and limitations

Advantages, stated narrowly:

- The KEM is isogeny-based, which is a different assumption family from
  the lattice KEMs NIST has standardised. Diversity is not a security proof.
- The five primes are regenerated by a public smallest-cofactor rule.
- The library crate is `#![forbid(unsafe_code)]`.
- Sizes and the cost model are computed from the same constants the KEM
  uses, and tests pin the cost-model floors.
- L5, Q3, and Q5 are sized so the known mask-guess Grover attack is at
  least 2^λ queries in that model.

Limitations:

- L1 and L3 fail a 2^λ quantum-query bar. Recommending them as Category 1
  or 3 is a false submission.
- Eigen-CIST and secret-degree EndRing are assumptions.
- The FO argument is ROM, author-written, and not externally reviewed.
  QROM IND-CCA is not claimed. `δ_corr = 0` is not proved.
- A mask-free dimension-4 attack is open (`docs/SECURITY.md` §8).
- Public keys and ciphertexts are a few kilobytes, and operations are
  seconds to hours (§9). That is a deployment limitation even where the
  query claim is the one being made.
- Constant-time behaviour of decapsulation is an implementation goal
  documented by the authors. It is not a third-party evaluation of this
  source.
- There is no ANSI C reference implementation.
- No outside cryptanalysis of the scheme has been recorded.

## 10. Performance

These are previously recorded measurements, not a new run for this draft
and not a NIST reference-platform report.

From the repository README (Intel Xeon at 2.80 GHz, one thread, run
`run-20261011T1813Z-bench`): L1 keygen 32.3 s (one sample), encaps about
0.25 s, decaps about 13–15 s. L3 keygen 118.6 s, encaps about 1 s, decaps
about 52–54 s. L5 keygen 271.8 s, encaps about 2.2 s, decaps about 120–124 s.

From the README (Apple laptop, 2026-10-12): Q3 keygen 1451 s, encaps 9.0 s,
decaps 547 s. From `research/q5_roundtrip.md` (2026-10-13, `--jobs 8` for
keygen only): Q5 keygen 9825 s, encaps 93 s, decaps 214 s.

Key generation is rejection sampling. The sample counts above are small.
[`GAPS.md`](GAPS.md) keeps L5/Q keygen performance open.

## 11. Intellectual property (placeholder)

This section does not claim a patent, and it does not claim that a patent
search was done.

- No patent is asserted on Eigen-Séta by this draft.
- No patent search has been performed. Silence is not a representation that
  no third-party patent exists.
- The signed owner statement required by the call is not attached. The
  cover note's submitter block is blank.
- The crate is under the MIT license in `LICENSE` (copyright 2026 Adam
  Buran). That copyright line is not a completed NIST intellectual-property
  statement.
- `src/kani.rs` includes Richelot / gluing formulas under
  `LICENSE.castryck-decru` (copyright 2023 Giacomo Pope, Rémy Oudompheng,
  Lorenz Panny).
- `src/theta.rs` is a port of the level-2 theta chain from
  Theta-Rust / `two-isogenies` (Pierrick Dartois, Luciano Maino, Giacomo
  Pope, Damien Robert), under `LICENSE.theta-isogenies`.
- Other dependencies are declared in `Cargo.toml` and keep their own
  licenses. This draft does not audit them.

Replace this section before any upload. Do not invent patent numbers.
