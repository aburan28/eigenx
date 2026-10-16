# Gaps before a real NIST upload

Nothing in this list is done. A check box here would be a false status.

The draft in this directory is an outline plus a short L1 known-answer file.
It is not an upload package.

## Required by the claim boundary

- [ ] External review of `docs/FO_PROOF.md`. The note is author-checked
      against `src/kem.rs` only.
- [ ] A proof that the correctness error is zero (`δ_corr = 0`) for every
      honest key and every kernel scalar, at every parameter set. Key
      generation self-tests one encapsulation, and one L1 test recovers
      several scalars. That is not the proof.
- [ ] Constant-time evidence on this source, produced and reviewed for the
      submission. `SIDE_CHANNELS.md` is an author assessment.
      `examples/dudect_decaps.rs` is a probe harness. Neither closes this
      item.
- [ ] L5, Q3, and Q5 key generation fast enough to document as a submission
      performance result on the NIST reference platform. Existing wall times
      (historical L5 keygen on the order of minutes; Q5 keygen on the order
      of hours with `--jobs 8`, see `research/q5_roundtrip.md`) show the
      cost. They do not close it.
- [ ] A mask-free dimension-4 (or dimension-8) attack ruled out, or the
      security claim revised if the subgroups `φ(⟨P⟩)` and `φ(⟨Q⟩)` suffice
      for a Kani embedding without guessing masks. The with-masks path is
      scaffolded. It does not remove the mask guess. See `docs/SECURITY.md`
      §8 and `research/kani4_sage/`.

## Required by the NIST call and still absent

- [ ] Signed cover sheet: principal submitter, auxiliary submitters, owner,
      inventors, backup contact.
- [ ] Completed intellectual-property statement, including a patent search.
      The specification has a placeholder. It claims no patent and does not
      assert that none exists.
- [ ] ANSI C reference implementation.
- [ ] Optimized implementation, and performance numbers on the NIST reference
      platform (AES-NI, SHA extensions, the call's stated CPU).
- [ ] Known-answer files produced with NIST's AES-256-CTR DRBG. This draft
      uses the repository's SHAKE256 DRBG and says so. The byte strings will
      not match a NIST-DRBG run.
- [ ] A full KAT count (the call's example generators use 100). The checked-in
      L1 set is a short count. L3, L5, Q3, and Q5 have no KAT files in this
      draft.
- [ ] A statement of expected strength that a third party has checked against
      the implementation. The numbers in the specification are the in-tree
      cost model.

## Claims that stay off the upload

- [ ] QROM IND-CCA. Not claimed. Do not add it.
- [ ] A reduction for eigen-CIST. It is an assumption.
- [ ] A reduction for secret-degree EndRing. It is an assumption.
- [ ] Recommending L1 as NIST Category 1, or L3 as NIST Category 3. That
      recommendation is a false submission. The Tier A gate-count table does
      not change this.
- [ ] Toy16 or Toy32 as a submitted parameter set. They were removed. Do not
      bring them back.

## Also still open in the analysis

- [ ] External cryptanalysis. `docs/SECURITY.md` §8 lists attacks that are
      costed and attacks that are not ruled out, including level-structure
      attacks and a distinguisher for decisional eigen-CIST.
- [ ] A written proof that the fixed decapsulation schedule is correct for
      every admissible secret degree, not only for the keys that pass the
      self-test.
