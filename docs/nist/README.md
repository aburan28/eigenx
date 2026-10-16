# Eigen-Séta draft NIST PQC package

**This directory is a draft. It is not ready to upload.**

A submission that recommends L1 or L3 as NIST Category 1 or 3 is a false
submission. The cover note and the specification both say so.

| NIST call item | Draft file | State |
|---|---|---|
| Cover sheet | [`COVER.md`](COVER.md) | Placeholder. Not signed. |
| Written specification (keygen, encaps, decaps), parameter sizes, security claims, KATs, rationale, advantages and limitations, IP | [`SPECIFICATION.md`](SPECIFICATION.md) | Draft, tied to this tree. |
| Known-answer values | [`kat/`](kat/) | L1, short count, SHAKE256 DRBG. Not the NIST AES-256-CTR DRBG. |
| Gap list before a real upload | [`GAPS.md`](GAPS.md) | Open items. None of them are done. |
| ANSI C reference and optimized implementations | — | Absent. See the gap list. |

The security claims are the ones already in [`../SECURITY.md`](../SECURITY.md)
and the repository README. This package does not add a stronger claim.

Generate or extend KATs with the `eigenx kat` command documented in the
specification. The checked-in L1 files are a short count so a reviewer can
see the format. They are not a complete NIST KAT set.
