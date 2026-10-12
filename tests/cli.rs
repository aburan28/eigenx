//! End-to-end test of the `eigenx` command-line tool at Toy16: keygen,
//! encaps/decaps, encrypt/decrypt, associated-data mismatch, info.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_eigenx"))
}

fn tmpdir() -> PathBuf {
    let d = std::env::temp_dir().join(format!("eigenx-cli-test-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run(args: &[&str]) -> (bool, String, String) {
    let out = bin().args(args).output().expect("spawn eigenx");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn cli_round_trip_toy16() {
    let d = tmpdir();
    let p = |n: &str| d.join(n).to_string_lossy().into_owned();
    let (ok, out, _) = run(&["params"]);
    assert!(ok);
    assert!(out.contains("Toy16") && out.contains("L5") && out.contains("Q5"));

    let (ok, _, err) = run(&[
        "keygen",
        "--set",
        "Toy16",
        "--pk",
        &p("pk"),
        "--sk",
        &p("sk"),
    ]);
    assert!(ok, "keygen failed: {err}");

    let (ok, _, err) = run(&[
        "encaps",
        "--pk",
        &p("pk"),
        "--ct",
        &p("ct"),
        "--key",
        &p("k1"),
    ]);
    assert!(ok, "encaps failed: {err}");
    let (ok, _, err) = run(&[
        "decaps",
        "--sk",
        &p("sk"),
        "--ct",
        &p("ct"),
        "--key",
        &p("k2"),
    ]);
    assert!(ok, "decaps failed: {err}");
    let k1 = std::fs::read(p("k1")).unwrap();
    let k2 = std::fs::read(p("k2")).unwrap();
    assert_eq!(k1, k2);
    assert_eq!(k1.len(), 8 + 32);

    std::fs::write(p("msg"), b"EigenX end-to-end message").unwrap();
    let (ok, _, err) = run(&[
        "encrypt",
        "--pk",
        &p("pk"),
        "--in",
        &p("msg"),
        "--out",
        &p("enc"),
        "--aad",
        "ctx",
    ]);
    assert!(ok, "encrypt failed: {err}");
    let (ok, _, err) = run(&[
        "decrypt",
        "--sk",
        &p("sk"),
        "--in",
        &p("enc"),
        "--out",
        &p("dec"),
        "--aad",
        "ctx",
    ]);
    assert!(ok, "decrypt failed: {err}");
    assert_eq!(
        std::fs::read(p("dec")).unwrap(),
        b"EigenX end-to-end message"
    );
    // Wrong associated data must fail.
    let (ok, _, err) = run(&[
        "decrypt",
        "--sk",
        &p("sk"),
        "--in",
        &p("enc"),
        "--out",
        &p("dec2"),
        "--aad",
        "other",
    ]);
    assert!(!ok);
    assert!(err.contains("decryption failed"));
    // Wrong object kind must fail with a clear message.
    let (ok, _, err) = run(&[
        "decaps",
        "--sk",
        &p("pk"),
        "--ct",
        &p("ct"),
        "--key",
        &p("k3"),
    ]);
    assert!(!ok);
    assert!(err.contains("expected a secret key"), "{err}");
    // info
    let (ok, out, _) = run(&["info", &p("enc")]);
    assert!(ok);
    assert!(
        out.contains("PKE ciphertext") && out.contains("message bytes: 25"),
        "{out}"
    );
    let (ok, out, _) = run(&["info", &p("sk")]);
    assert!(ok);
    assert!(
        out.contains("secret key") && !out.contains("warning"),
        "{out}"
    );
    std::fs::remove_dir_all(&d).ok();
}
