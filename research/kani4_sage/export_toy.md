# Exporting Toy16 (pk, ct) for the Sage driver

Until a dedicated exporter lands, use known-answer vectors or a one-off Rust
snippet.

## From KATs

```text
# payloads are raw field encodings (no EIGX header)
xxd -p tests/kat/toy16_pk.bin   # if present; else see tests/kat/
```

## From a short Rust probe

```rust
// cargo test --release --example … or a throwaway main
use eigen_seta::kem::EigenSeta;
use eigen_seta::params::Toy16;
use rand_core::OsRng;
let (pk, _sk) = EigenSeta::<Toy16, 4>::keygen(&mut OsRng);
let (ct, _) = EigenSeta::<Toy16, 4>::encaps(&pk, &mut OsRng).unwrap();
println!("pk {}", hex::encode(&pk.bytes));
println!("ct {}", hex::encode(&ct.bytes));
```

Pass the hex strings to `sage -c "load('attack.sage'); run_from_hex(pk, ct)"`
after `THETA_DIM4` is set (see README.md).
