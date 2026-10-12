//! `eigenx`: command-line front end for the Eigen-Séta KEM and PKE.
//!
//! ```text
//! eigenx params
//! eigenx keygen  --set L1 --pk pk.eigx --sk sk.eigx
//! eigenx encaps  --pk pk.eigx --ct ct.eigx --key key.eigx
//! eigenx decaps  --sk sk.eigx --ct ct.eigx --key key.eigx
//! eigenx encrypt --pk pk.eigx --in message.txt --out message.eigx [--aad context]
//! eigenx decrypt --sk sk.eigx --in message.eigx --out message.txt [--aad context]
//! eigenx info    FILE
//! ```
//!
//! All objects are stored in the container format of `eigen_seta::wire`.
//! This is research software: the scheme's security is conjectural
//! (see docs/SECURITY.md).  Do not use it to protect real data.

use eigen_seta::kem::{Ciphertext, EigenSeta, PublicKey};
use eigen_seta::params::ParamSet;
use eigen_seta::pke::{EigenSetaPke, PkeCiphertext};
use eigen_seta::sets::set_info;
use eigen_seta::wire::{self, Kind, SetId};
use eigen_seta::with_set;
use rand_core::OsRng;
use std::collections::HashMap;
use std::process::ExitCode;
use zeroize::Zeroize;

const USAGE: &str = "\
eigenx: Eigen-Séta KEM / PKE command-line tool (research software)

  eigenx params
  eigenx keygen  --set <Toy16|Toy32|L1|L3|L5|Q3|Q5> --pk FILE --sk FILE
  eigenx encaps  --pk FILE --ct FILE --key FILE
  eigenx decaps  --sk FILE --ct FILE --key FILE
  eigenx encrypt --pk FILE --in FILE --out FILE [--aad STRING]
  eigenx decrypt --sk FILE --in FILE --out FILE [--aad STRING]
  eigenx info    FILE
";

type Res<T> = Result<T, String>;

fn parse_flags(args: &[String]) -> Res<HashMap<String, String>> {
    let mut m = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        let Some(name) = a.strip_prefix("--") else {
            return Err(format!("unexpected argument `{a}`"));
        };
        let Some(v) = args.get(i + 1) else {
            return Err(format!("flag --{name} needs a value"));
        };
        if m.insert(name.to_string(), v.clone()).is_some() {
            return Err(format!("flag --{name} given twice"));
        }
        i += 2;
    }
    Ok(m)
}

fn need<'a>(m: &'a HashMap<String, String>, k: &str) -> Res<&'a str> {
    m.get(k)
        .map(|s| s.as_str())
        .ok_or_else(|| format!("missing --{k}"))
}

fn read(path: &str) -> Res<Vec<u8>> {
    std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))
}

fn write(path: &str, data: &[u8]) -> Res<()> {
    std::fs::write(path, data).map_err(|e| format!("cannot write {path}: {e}"))
}

fn load(path: &str, want: Kind) -> Res<(SetId, Vec<u8>)> {
    let bytes = read(path)?;
    let (h, payload) = wire::decode(&bytes).map_err(|e| format!("{path}: {e}"))?;
    if h.kind != want {
        return Err(format!(
            "{path}: expected a {}, found a {}",
            want.name(),
            h.kind.name()
        ));
    }
    Ok((h.set, payload.to_vec()))
}

fn cmd_params() -> Res<()> {
    println!(
        "{:<6} {:>6} {:>7} {:>5} {:>5} {:>4} {:>8} {:>7} {:>6} {:>6} {:>7}  quantum-hardened",
        "set", "lambda", "p_bits", "limbs", "E", "F3", "log2(d)", "f_bits", "pk_B", "ct_B", "sk_B",
    );
    for id in SetId::ALL {
        let s = set_info(id);
        println!(
            "{:<6} {:>6} {:>7} {:>5} {:>5} {:>4} {:>8.1} {:>7} {:>6} {:>6} {:>7}  {}",
            s.name,
            s.lambda,
            s.p_bits,
            s.limbs,
            s.e,
            s.f3,
            s.d_bits,
            s.f_bits,
            s.pk_bytes,
            s.ct_bytes,
            s.sk_bytes,
            if s.quantum_hardened { "yes" } else { "no" }
        );
    }
    Ok(())
}

fn cmd_keygen(m: &HashMap<String, String>) -> Res<()> {
    let set_name = need(m, "set")?;
    let id = SetId::from_name(set_name).ok_or_else(|| format!("unknown set `{set_name}`"))?;
    let (pk_path, sk_path) = (need(m, "pk")?, need(m, "sk")?);
    let mut rng = OsRng;
    with_set!(id, P, N, {
        let (pk, sk) = EigenSeta::<P, N>::keygen(&mut rng);
        let mut skb = EigenSeta::<P, N>::sk_to_bytes(&sk);
        let out = wire::encode(Kind::SecretKey, id, &skb);
        skb.zeroize();
        write(sk_path, &out)?;
        write(pk_path, &wire::encode(Kind::PublicKey, id, &pk.bytes))?;
        eprintln!(
            "{}: wrote public key ({} B payload) and secret key ({} B payload)",
            <P as ParamSet<N>>::NAME,
            pk.bytes.len(),
            out.len() - wire::HEADER_LEN
        );
    });
    Ok(())
}

fn cmd_encaps(m: &HashMap<String, String>) -> Res<()> {
    let (id, pkb) = load(need(m, "pk")?, Kind::PublicKey)?;
    let (ct_path, key_path) = (need(m, "ct")?, need(m, "key")?);
    let mut rng = OsRng;
    with_set!(id, P, N, {
        let pk = PublicKey { bytes: pkb };
        let (ct, mut key) =
            EigenSeta::<P, N>::encaps(&pk, &mut rng).ok_or("public key failed validation")?;
        write(ct_path, &wire::encode(Kind::KemCiphertext, id, &ct.bytes))?;
        let r = write(key_path, &wire::encode(Kind::SharedSecret, id, &key));
        key.zeroize();
        r?;
    });
    Ok(())
}

fn cmd_decaps(m: &HashMap<String, String>) -> Res<()> {
    let (id, mut skb) = load(need(m, "sk")?, Kind::SecretKey)?;
    let (id2, ctb) = load(need(m, "ct")?, Kind::KemCiphertext)?;
    if id != id2 {
        skb.zeroize();
        return Err("secret key and ciphertext use different parameter sets".into());
    }
    let key_path = need(m, "key")?;
    with_set!(id, P, N, {
        let sk = EigenSeta::<P, N>::sk_from_bytes(&skb);
        skb.zeroize();
        let sk = sk.ok_or("secret key file is malformed")?;
        let mut key = EigenSeta::<P, N>::decaps(&sk, &Ciphertext { bytes: ctb });
        let r = write(key_path, &wire::encode(Kind::SharedSecret, id, &key));
        key.zeroize();
        r?;
    });
    Ok(())
}

fn cmd_encrypt(m: &HashMap<String, String>) -> Res<()> {
    let (id, pkb) = load(need(m, "pk")?, Kind::PublicKey)?;
    let msg = read(need(m, "in")?)?;
    let out_path = need(m, "out")?;
    let aad = m.get("aad").map(|s| s.as_bytes()).unwrap_or(b"");
    let mut rng = OsRng;
    with_set!(id, P, N, {
        let pk = PublicKey { bytes: pkb };
        let ct = EigenSetaPke::<P, N>::encrypt(&pk, &msg, aad, &mut rng)
            .ok_or("public key failed validation")?;
        write(out_path, &wire::encode(Kind::PkeCiphertext, id, &ct.bytes))?;
    });
    Ok(())
}

fn cmd_decrypt(m: &HashMap<String, String>) -> Res<()> {
    let (id, mut skb) = load(need(m, "sk")?, Kind::SecretKey)?;
    let (id2, ctb) = load(need(m, "in")?, Kind::PkeCiphertext)?;
    if id != id2 {
        skb.zeroize();
        return Err("secret key and ciphertext use different parameter sets".into());
    }
    let out_path = need(m, "out")?;
    let aad = m.get("aad").map(|s| s.as_bytes()).unwrap_or(b"");
    with_set!(id, P, N, {
        let sk = EigenSeta::<P, N>::sk_from_bytes(&skb);
        skb.zeroize();
        let sk = sk.ok_or("secret key file is malformed")?;
        let pt = EigenSetaPke::<P, N>::decrypt(&sk, &PkeCiphertext { bytes: ctb }, aad)
            .ok_or("decryption failed: ciphertext, key or associated data do not match")?;
        write(out_path, &pt)?;
    });
    Ok(())
}

fn cmd_info(path: &str) -> Res<()> {
    let bytes = read(path)?;
    let (h, payload) = wire::decode(&bytes).map_err(|e| format!("{path}: {e}"))?;
    let s = set_info(h.set);
    let expected = match h.kind {
        Kind::PublicKey => Some(s.pk_bytes),
        Kind::SecretKey => Some(s.sk_bytes),
        Kind::KemCiphertext => Some(s.ct_bytes),
        Kind::SharedSecret => Some(eigen_seta::kem::SS_BYTES),
        Kind::PkeCiphertext => None,
    };
    println!("file:          {path}");
    println!("kind:          {}", h.kind.name());
    println!(
        "parameter set: {} (lambda = {}, p has {} bits)",
        s.name, s.lambda, s.p_bits
    );
    println!("payload bytes: {}", payload.len());
    match expected {
        Some(n) if n != payload.len() => {
            println!("warning:       expected {n} payload bytes for this kind");
        }
        None => {
            let min = s.ct_bytes + eigen_seta::pke::NONCE_BYTES + eigen_seta::pke::TAG_BYTES;
            if payload.len() >= min {
                println!("message bytes: {}", payload.len() - min);
            } else {
                println!("warning:       too short for a PKE ciphertext (minimum {min})");
            }
        }
        _ => {}
    }
    Ok(())
}

fn run(args: &[String]) -> Res<()> {
    let Some(cmd) = args.first() else {
        return Err(USAGE.to_string());
    };
    let rest = &args[1..];
    match cmd.as_str() {
        "params" => cmd_params(),
        "keygen" => cmd_keygen(&parse_flags(rest)?),
        "encaps" => cmd_encaps(&parse_flags(rest)?),
        "decaps" => cmd_decaps(&parse_flags(rest)?),
        "encrypt" => cmd_encrypt(&parse_flags(rest)?),
        "decrypt" => cmd_decrypt(&parse_flags(rest)?),
        "info" => match rest {
            [p] => cmd_info(p),
            _ => Err("info takes exactly one FILE".into()),
        },
        "help" | "--help" | "-h" => {
            print!("{USAGE}");
            Ok(())
        }
        other => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
