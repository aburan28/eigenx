# IND-CCA security of Eigen-Séta via the FO transform

This note proves that the KEM in `src/kem.rs` is IND-CCA secure in the
random-oracle model, under the two computational assumptions stated in
`docs/SECURITY.md` §6, up to a correctness-error term that is made explicit.
It follows the modular framework of Hofheinz–Hövelmanns–Kiltz (HHK,
EUROCRYPT 2017 / ePrint 2015/917), specialised to the implicit-rejection
transform used here.

**Status.** Author-written. Not peer-reviewed. The reduction is standard; the
novelty, if any, is only in naming the Eigen-Séta trapdoor assumptions and
recording the concrete domain separations from the implementation.

## 1. Syntax

### 1.1 Trapdoor one-way function

Public parameters `pp` include the parameter set (p, E, F3, …) and the public
CM(−7) curve E_D. Algorithms:

* `KeyGen(pp; r) → (pk, sk)` — as in `EigenSeta::keygen`.
* `Tof(pk, s) → ct` — deterministic encapsulation of a kernel scalar
  `s ∈ ℤ/dℤ` (`enc_det` / `enc_det_core`). Output length `ct_bytes`.
* `Inv(sk, ct) → s ∪ {⊥}` — the public-prepare + constant-time core that
  recovers `s` on honest ciphertexts and returns an unspecified value on
  malformed ones; the KEM wraps it with re-encryption checking (below).

Correctness: for every honestly generated `(pk, sk)` and every `s`,

```
Inv(sk, Tof(pk, s)) = s
```

except on a set of keys of measure at most `δ_corr` (the *correctness error*).
In the implementation, every key that passes the self-test in `keygen_attempt`
satisfies exact correctness for all `s` tried; we have never observed a
failure on an honest ciphertext at any parameter set. The FO bound below
carries `δ_corr` explicitly; empirically `δ_corr = 0` on the generated keys.

### 1.2 Derandomised FO with implicit rejection

Let `P, G, K, R` be independent random oracles (instantiated by SHAKE256 with
domain-separation prefixes `EigenSeta-P`, `-G`, `-K`, `-R` as in `kem.rs`).

```
Encaps(pk):
  s ← ℤ/dℤ
  pkh ← P(pk)
  ct  ← Tof(pk, s)           // uses G(s, pkh) as masks
  K   ← K(s, ct)
  return (ct, K)

Decaps(sk, ct):
  // sk contains (trapdoor, pk, z) with z ← {0,1}^256
  s'  ← Inv(sk, ct)
  ct' ← Tof(pk, s')
  if ct' = ct: return K(s', ct)
  else:         return R(z, ct)
```

Malformed public keys are rejected by `Encaps` before any oracle call
(`pk_points_valid`); they are out of scope for the CCA game against honest
`pk`.

## 2. Assumptions

**OW-CPA for Tof (eigen-CIST).** For every PPT adversary A,

```
Adv^ow-cpa_Tof(A) := Pr[ A(pk, Tof(pk, s)) = s ]
```

is negligible, where `(pk, sk) ← KeyGen` and `s ← ℤ/dℤ`. This is the
computational form of decisional eigen-CIST (`SECURITY.md` §6.1): recovering
the sender's 3^F3-isogeny from the masked torsion images. Best known attacks
are costed in `SECURITY.md` §3.1.

**Key secrecy (secret-degree EndRing).** For every PPT adversary A,

```
Adv^ks_KeyGen(A) := Pr[ A(pk) outputs sk' that Decaps some honest ct correctly ]
```

is negligible. Equivalent (up to the two sign choices of the Kani scalars) to
recovering the secret degree f (`SECURITY.md` §6.2, claim C1).

We do not need a separate assumption on the masks G: they are derived from s
by a random oracle, so an adversary that predicts them without s breaks OW-CPA
directly.

## 3. Theorem

**Theorem (ROM IND-CCA).** Let E be an IND-CCA adversary against the KEM that
makes at most `q_D` decryption queries, `q_P, q_G, q_K, q_R` queries to the
respective random oracles, and runs in time t. Then there exist OW-CPA
adversary B and key-secrecy adversary C, each running in time t + poly(λ) ·
(q_D + q_P + q_G + q_K + q_R), such that

```
Adv^ind-cca_KEM(E)
  ≤  Adv^ow-cpa_Tof(B)
   + Adv^ks_KeyGen(C)
   + q_D · δ_corr
   + (q_K + q_R + q_D) / 2^256
   + q_G · 2^{1 − E}
   + negl_collision.
```

The term `2^{1−E}` bounds the chance that two distinct odd masks collide mod
2^E after the `| = 1` forcing in `masks`; `negl_collision` accounts for
SHAKE256 collisions on the 32-byte digests (birthday, ≤ q²/2^256).

### 3.1 Proof sketch (HHK)

We follow HHK's game hop for the *implicit-rejection*, *derandomised*,
*injective* FO transform. The Eigen-Séta particulars are only the concrete
oracles and the form of `Tof`.

**Game 0.** The IND-CCA game.

**Game 1 (honesty of pk).** The challenger rejects if `pk` fails
`pk_points_valid`. Negligible difference: KeyGen always produces a valid pk.

**Game 2 (unique decryption for honest keys).** On each decryption query ct,
the challenger computes s' ← Inv(sk, ct) and ct' ← Tof(pk, s'). If ct' ≠ ct
but Inv would have returned a value that Tof maps back to ct under a different
preimage, abort. Difference ≤ `q_D · δ_corr` by correctness, plus a random-oracle
collision term if two s values hash to masks producing the same ct (bounded by
`q_G · 2^{1−E}` because a, b are odd in (ℤ/2^Eℤ)×).

**Game 3 (implicit rejection is independent).** Replace `R(z, ct)` by a
lazy-sampled random 32-byte string per ct, independent of K. Difference ≤
`(q_R + q_D)/2^256` (z is uniform and hidden; R is a RO).

**Game 4 (OW-CPA reduction).** Let ct* be the challenge ciphertext, encapsulating
uniform s*. An adversary that queries K(s*, ct*) wins OW-CPA by outputting s*.
Decryption queries for ct ≠ ct* are answered using sk (or the implicit-rejection
table). A decryption query for ct* returns the challenge key. The HHK analysis
shows that any event in which the CCA adversary distinguishes Game 4 from a
random key yields either a query K(s*, ct*) (OW-CPA break) or a forgery of a
second preimage under Tof (key secrecy / correctness). Formally one builds B
that runs E, simulates the ROs, and on the first query of the form K(s, ct*)
outputs s; Adv(E) ≤ Adv(B) + Adv(C) + the additive terms above.

**Injectivity of Tof.** Needed by HHK so that each ct has at most one s. Here
Tof(pk, ·) is injective for honest pk except with probability ≤ 2^{1−E} over the
masks of two distinct s (the 3-isogeny kernel determines E_C uniquely, and the
masked torsion images determine (a,b)·φ(P,Q) ; distinct odd (a,b) give distinct
images except on a 2-torsion coincidence of measure 2^{O(1)−E}).

## 4. QROM

The reduction above is ROM, not QROM. A QROM lift of implicit-rejection FO
exists under additional hypotheses (e.g. Bindel–Hamburg–Hövelmanns–Hülsing /
Don–Fehr–Majenz–Schäffner) at the cost of a square-root loss in the RO query
bound. We do **not** claim QROM IND-CCA for the L sets; the quantum-hardened
sets Q3/Q5 are sized so that even the OW-CPA attack (mask-guess under Grover)
costs ≥ 2^λ quantum queries (`SECURITY.md` §5), which is the relevant figure
if one only assumes quantum OW-CPA.

## 5. What this does not prove

* Hardness of eigen-CIST or secret-degree EndRing (taken as assumptions).
* Tightness against multi-target or key-dependent-message attacks.
* Security of the KEM-DEM PKE in `pke.rs` beyond the standard one-time
  authenticated-encryption composition (ChaCha20-Poly1305 is assumed AEAD).
* Side-channel security: the reduction assumes the implementation matches the
  algorithm; constant-time discipline is argued separately in `SIDE_CHANNELS.md`.

## 6. Checklist against the code

| HHK ingredient | Implementation |
|---|---|
| Derandomised Tof | `enc_det` / `enc_det_core` |
| Injectivity (odd masks) | `masks`: `a.l[0] \|= 1`, `b.l[0] \|= 1` |
| Re-encryption check | `decaps_core`: `bytes_eq(ct2, ct)` |
| Implicit rejection | `R(z, ct)` via `shake([EigenSeta-R, z, ct])` |
| Independent keying RO | `K(s, ct)` via `shake([EigenSeta-K, s, ct])` |
| pk hash into G | `pkh = P(pk)`, `G(s, pkh)` |
| Fresh z per key | `SecretKey.z` from RNG at keygen |

## 7. References

* D. Hofheinz, K. Hövelmanns, E. Kiltz, *A Modular Analysis of the Fujisaki–Okamoto Transformation*, TCC 2017, ePrint 2015/917.
* E. Fujisaki, T. Okamoto, *Secure Integration of Asymmetric and Symmetric Encryption Schemes*, CRYPTO 1999 / J. Cryptology 2013.
* docs/SECURITY.md §§6–8 (assumptions, open questions on correctness failure rate).
