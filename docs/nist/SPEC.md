# Eigen-Séta KEM specification (draft)

Read from `src/kem.rs`, `src/wire.rs`, and `src/params_generated.rs`.
This note describes the five parameter sets L1, L3, L5, Q3, and Q5.
Toy sets are not part of the scheme. Shared secret length is 32 bytes
(`SS_BYTES`). The crate forbids `unsafe` (`#![forbid(unsafe_code)]` in
`src/lib.rs`).

File objects written by the `eigenx` tool add an 8-byte header
(`b"EIGX" || version || kind || set id || 0`) defined in `src/wire.rs`.
The byte counts below are the raw KEM objects, without that header.
Set ids: L1 = 3, L3 = 4, L5 = 5, Q3 = 7, Q5 = 8. The name `Q1` decodes
as L5. There is no set id 6.

## Parameter sizes

`p = 2^{E+1} · 3^{F3} · c − 1`, `d = 3^{F3}`. Public key and ciphertext
are each four `F_{p^2}` elements. One `F_p` element is
`ceil(bitlength(p)/8)` bytes, so

`pk_bytes = ct_bytes = 8 · ceil(bitlength(p)/8)`.

A secret key is three scalars modulo `2^E`, two scalars modulo `d`, a
32-byte rejection seed `z`, the 8-byte little-endian bit length of the
secret degree, and the public key (`EigenSeta::sk_bytes`):

`sk_bytes = 3·ceil(E/8) + 2·ceil(d_bits/8) + 40 + pk_bytes`,

where `d_bits` is the bit length of `d`. Encapsulation draws

`seed_len = ceil(d_bits/8) + ceil(λ/8)`

bytes and reduces them modulo `d` (`EigenSeta::encaps`).

| Set | λ | p bits | limbs | E | F3 | c | d bits | pk = ct | sk | ss | encaps seed |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| L1 | 128 | 1341 | 21 | 1076 | 162 | 144 | 257 | 1344 | 1855 | 32 | 49 |
| L3 | 192 | 1991 | 32 | 1594 | 243 | 1059 | 386 | 1992 | 2730 | 32 | 73 |
| L5 (= Q1) | 256 | 2630 | 42 | 2106 | 324 | 380 | 514 | 2632 | 3594 | 32 | 97 |
| Q3 | 192 | 3904 | 61 | 3125 | 485 | 371 | 769 | 3904 | 5311 | 32 | 121 |
| Q5 | 256 | 5192 | 82 | 4155 | 647 | 975 | 1026 | 5192 | 7050 | 32 | 161 |

κ = 12 on every set. L sets use `d ≥ 2^{2λ}`. Q3 and Q5 use `d ≥ 2^{4λ}`.
L5 is the quantum-hardened λ = 128 set; the generator does not emit a
second prime for Q1.

## Key generation

`EigenSeta::keygen` is variable time and is meant to run where the
secret degree may be handled in the clear (`SIDE_CHANNELS.md`).

1. Prepare the public CM(−7) curve, a root of −7 modulo `2^E`, and the
   eigenvalue split used by the Kani endomorphism (`keygen_prep`).
2. Draw a secret odd degree `f` from `[f_lo, 2·f_lo)`, with `f_lo` the
   integer in `F_LO_HEX`. Reject `f` that is divisible by 2, 3, or 7,
   or that fails the Kani sign schedule (`kani_scalars_for_degree`).
3. Build the public Montgomery curve `E_A` and a scaled eigenbasis.
   The public key is four `F_{p^2}` elements: the curve coefficient and
   the x-coordinates of the two basis points and their difference.
4. The secret key stores the Kani scalars derived from `f`, not `f`
   itself, plus a fresh 32-byte seed `z` and a SHAKE256 hash of the
   public key under the domain `EigenSeta-P`.
5. **Self-test.** One honest encapsulation and decapsulation are run
   with the same RNG. If the shared secrets differ, the attempt is
   discarded and key generation retries (`keygen_attempt`). This is one
   sample. It is not a proof that the correctness error `δ_corr` is
   zero.

`keygen_with_jobs` runs several attempt loops and keeps the first
success. It does not change the distribution beyond the usual rejection
sampling.

## Encapsulation

`EigenSeta::encaps` returns `None` if the public key is not four field
elements or fails `pk_points_valid` (singular curve, or a basis that
does not match the recorded difference). Otherwise:

1. `pkh ← SHAKE256("EigenSeta-P" || pk)` truncated to 32 bytes.
2. Draw `seed_len` bytes. Reduce modulo `d = 3^{F3}` to a kernel
   scalar `s`. The extra `ceil(λ/8)` bytes bound the statistical
   distance of that reduction by `2^{-λ}`.
3. Derive odd masks `(a, b)` modulo `2^E` by
   `SHAKE256("EigenSeta-G" || s || pkh)`, forcing both least bits to 1.
4. Evaluate the deterministic trapdoor: a `3^{F3}`-isogeny and the
   masked images, encoded as four `F_{p^2}` elements (codomain
   coefficient and three x-coordinates). That encoding is the ciphertext.
5. `ss ← SHAKE256("EigenSeta-K" || s || ct)` truncated to 32 bytes.

Encapsulation is written to be constant time in `s` and the masks.
Parsing and basis checks depend only on the public key.

## Decapsulation

`EigenSeta::decaps` always returns 32 bytes.

1. **Public prepare** (`decaps_prepare`). Parse the ciphertext. Reject
   a singular codomain or points whose difference does not match the
   third x-coordinate. Lift points and compute the public `3^{F3}`
   bases. Failure here is public malformation.
2. **Secret core** (`decaps_core`). Recover a candidate `s` with the
   secret Kani scalars and the theta `(2,2)`-chain, re-encrypt with
   `enc_det_core`, and compare in constant time.
3. On a match, return `SHAKE256("EigenSeta-K" || s || ct)`.
   Otherwise return the **implicit rejection** key
   `SHAKE256("EigenSeta-R" || z || ct)`.
   The same rejection key is returned immediately when prepare fails.
   A caller cannot distinguish a malformed ciphertext from a
   re-encryption failure by the output length.

The theta chain is the decapsulation path. Mumford coordinates remain
for `inspect_chain` and the comparison test
`tests/theta_chain.rs::theta_chain_matches_mumford_l1`.

## What this draft does not specify

IND-CCA in the quantum random-oracle model is not claimed. The ROM
argument in `docs/FO_PROOF.md` is an author check against `kem.rs`.
Eigen-CIST and secret-degree EndRing are assumptions
(`docs/SECURITY.md` §6). Category language is in `SUBMISSION.md`.
