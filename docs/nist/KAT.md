# Known-answer tests (draft)

Two generators exist. Both use the SHAKE256 DRBG in `src/drbg.rs`
(`EigenSeta-DRBG`), not NIST AES-256-CTR `randombytes`. Neither assigns
L1 to NIST Category 1.

## Checked-in L1 vectors

`tests/kat/l1.txt` is one vector from `tests/kat.rs::kat_l1`. The DRBG
seeds are `Drbg::for_vector("L1-keygen", 0)`, `"L1-encaps"`, and
`"L1-pke"`. The file records raw `pk`, `sk`, `ct`, `ss`, and a PKE
ciphertext.

Check it (this runs L1 key generation):

```sh
CARGO_TARGET_DIR=/Volumes/data/eigenx-target \
RUSTFLAGS='-C target-cpu=native' \
cargo test --release --test kat kat_l1 -- --exact
```

Regenerate only when the algorithm or encoding changes, then review the diff:

```sh
CARGO_TARGET_DIR=/Volumes/data/eigenx-target \
RUSTFLAGS='-C target-cpu=native' \
EIGENX_UPDATE_KAT=1 \
cargo test --release --test kat kat_l1 -- --exact
```

## NIST-style request / response files

`src/kat_nist.rs` writes `PQCkemKAT_L1_{sk_bytes}.{req,int,rsp}`.
The master seed is the string `EigenSeta-NIST-KAT-v1-L1`. It emits 48-byte
seeds in order. Each vector runs `keygen` and then `encaps` on one DRBG
started from that seed. Key generation draws an extra encapsulation
inside its self-test; those bytes are `keygen_coins` in the `.int` file.
Encapsulation must draw `ceil(d_bits/8)+ceil(λ/8)` bytes (49 on L1).
The header includes the L1 claim line from `kat_nist::claim_line`: gate-count
Tier A only, not Category 1, Grover on mask-guess about `2^{λ/2}`.

Count 1 is one L1 key generation plus the self-test and one more
encaps/decaps. On the machines recorded in the README, one L1 keygen is
tens of seconds, so a count of 1 or 2 is expected to finish well under
15 minutes once the release binary is built. Count is not a multi-hour
search. Do not start this if another `cargo` is already using the same
target directory.

```sh
CARGO_TARGET_DIR=/Volumes/data/eigenx-target \
RUSTFLAGS='-C target-cpu=native' \
cargo run --release --bin eigenx -- kat --set L1 --count 1 --out docs/nist/kat
```

This draft already contains a count of 1, written by that command:

- `docs/nist/kat/PQCkemKAT_L1_1855.req`
- `docs/nist/kat/PQCkemKAT_L1_1855.int`
- `docs/nist/kat/PQCkemKAT_L1_1855.rsp`

Encapsulation seed length in that run was 49 bytes. The header states that
recommending L1 as NIST Category 1 is a false submission. Use `--count 2`
only to extend the set. The tool refuses counts outside 1..=10000. After
the files exist, decapsulation-only checking is:

```sh
CARGO_TARGET_DIR=/Volumes/data/eigenx-target \
RUSTFLAGS='-C target-cpu=native' \
cargo test --release --test kat nist_kat_l1_rsp_decaps_matches -- --exact
```

That test does not re-run key generation. It requires a
`docs/nist/kat/PQCkemKAT_L1_*.rsp` whose header contains the words
`false submission`.

L3, L5, Q3, and Q5 use the same command with `--set`. L5 may be passed
as `Q1`. Those key generations are slower (see `RATIONALE.md`) and are
not part of the short L1 vector this note asks for.
