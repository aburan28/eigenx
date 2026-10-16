# Design rationale (draft)

This note explains the choices in the current tree. It is not a security
proof and it is not an external review.

## Design

Eigen-Séta is an isogeny KEM. The public key is a curve `E_A` with a
basis. The secret is an endomorphism whose degree `f` is drawn from a
large interval and is not stored: decapsulation reconstructs what it
needs from Kani scalars that are a function of `f`
(`EigenSeta::secret_key_from_parts`). A party who learns `f` can rebuild
candidate secret keys (`secret_keys_from_degree`). The trapdoor
evaluation sends a kernel scalar `s ∈ Z/dZ`, `d = 3^{F3}`, through a
`3`-power isogeny and publishes masked images of the eigenbasis.

Encapsulation is derandomised Fujisaki–Okamoto with implicit rejection.
Domain-separated SHAKE256 instantiates the oracles `EigenSeta-P`,
`-G`, `-K`, and `-R`. Masks are forced odd. Decapsulation re-encrypts
and selects the shared secret or the rejection key with a constant-time
compare (`decaps_core`). Key generation is variable time and checks one
honest round trip before it returns a key.

The classical sets L1, L3, and L5 take `d ≥ 2^{2λ}`. Under Grover, a
mask-guess then costs about `2^{λ/2}` quantum queries, so those sets
are argued only at the NIST gate-count tier (Tier A in
`docs/SECURITY.md`). Q3 and Q5 take `d ≥ 2^{4λ}` so that the same
square root lands at `2^λ`. At λ = 128 that construction is exactly L5,
which is why Q1 is not a sixth prime.

Decapsulation walks a level-2 theta `(2^n, 2^n)`-chain
(`src/theta.rs`, `chain::decaps_chain_theta`), ported from the
Theta-Rust code under `LICENSE.theta-isogenies`. The field primes are
the nothing-up-my-sleeve primes generated into `params_generated.rs`.
This draft does not replace them.

## Advantages

- One Rust implementation covers key generation, encapsulation,
  decapsulation, secret-key encoding, and the file format. The library
  forbids unsafe code.
- Ciphertexts and public keys are four field elements. The shared
  secret is 32 bytes on every set.
- Implicit rejection does not return a special failure symbol, so a
  malformed ciphertext and a wrong plaintext look the same at the API.
- The cost model separates gate-count claims from quantum-query claims,
  and the KAT header repeats the L1/L3 warning (`src/kat_nist.rs`).
- Hostile-input tests check that bit flips and junk ciphertexts take
  the rejection path (`tests/robustness.rs`).

## Limitations

- External review has not been done. `docs/FO_PROOF.md` was checked by
  the authors against `src/kem.rs`. It has not been reviewed outside
  the project.
- `δ_corr` is not proved. Key generation self-tests a single honest
  ciphertext, and an L1 unit test recovers several kernel scalars.
  The FO bound still carries a `q_D · δ_corr` term. Empirically the
  self-test has not failed on generated keys; that is not a proof for
  every scalar and every set.
- L5, Q3, and Q5 key generation is still slow. Published samples in
  the README are on the order of minutes for L5 and about 1451 seconds
  for one Q3 key on a laptop core. Q5 has field tests and has not been
  timed end to end. Rejection sampling makes keygen time vary from key
  to key. Parallel attempts (`--jobs`) do not remove that cost.
- QROM IND-CCA is not claimed. The written reduction is in the
  classical random-oracle model.
- Eigen-CIST (message recovery) and secret-degree EndRing (key
  recovery) are assumptions. They are not reduced to a standard problem.
- L1 and L3 are not NIST Category 1 or 3 under a `2^λ` quantum-query
  bar. Grover on mask-guess is about `2^{λ/2}`.
- There is no ANSI C reference or optimized implementation in this
  tree. The DRBG used for known-answer tests is SHAKE256, not NIST
  AES-256-CTR `randombytes`.
- Key generation is not constant time. Side-channel claims for
  encapsulation and decapsulation are by construction review, not a
  third-party lab report (`SIDE_CHANNELS.md`).

## Intellectual property

Placeholder only. This draft does not assert, list, or invent any
patent, patent application, or patent number. No patent search was
performed for this package. Copyright and the third-party licence for
the theta-isogeny code are the `LICENSE`, `LICENSE.castryck-decru`,
and `LICENSE.theta-isogenies` files in the repository. Those licences
are not a patent statement. Names of submitters, owners, and an
inventors' affidavit are not filled in.
