//! Versioned container format for keys, ciphertexts and shared secrets as
//! written by the `eigenx` command-line tool.
//!
//! Layout: `b"EIGX"` || version (1 byte, currently 1) || kind (1 byte) ||
//! parameter-set id (1 byte) || reserved (1 byte, 0) || payload.  The
//! payload is the raw object: the KEM's public key, the serialised secret
//! key, the KEM or PKE ciphertext, or the 32-byte shared secret.

pub const MAGIC: &[u8; 4] = b"EIGX";
pub const VERSION: u8 = 1;
pub const HEADER_LEN: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    PublicKey = 1,
    SecretKey = 2,
    KemCiphertext = 3,
    PkeCiphertext = 4,
    SharedSecret = 5,
}

impl Kind {
    pub fn from_u8(b: u8) -> Option<Kind> {
        Some(match b {
            1 => Kind::PublicKey,
            2 => Kind::SecretKey,
            3 => Kind::KemCiphertext,
            4 => Kind::PkeCiphertext,
            5 => Kind::SharedSecret,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Kind::PublicKey => "public key",
            Kind::SecretKey => "secret key",
            Kind::KemCiphertext => "KEM ciphertext",
            Kind::PkeCiphertext => "PKE ciphertext",
            Kind::SharedSecret => "shared secret",
        }
    }
}

/// Parameter-set identifiers.  Stable: never renumber.  There is no `Q1`:
/// the quantum-hardened 128-bit set coincides with `L5` (both need
/// d = 3^324), so `L5` serves both roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SetId {
    Toy16 = 1,
    Toy32 = 2,
    L1 = 3,
    L3 = 4,
    L5 = 5,
    Q3 = 7,
    Q5 = 8,
}

impl SetId {
    pub const ALL: [SetId; 7] = [
        SetId::Toy16,
        SetId::Toy32,
        SetId::L1,
        SetId::L3,
        SetId::L5,
        SetId::Q3,
        SetId::Q5,
    ];
    pub fn from_u8(b: u8) -> Option<SetId> {
        SetId::ALL.iter().copied().find(|s| *s as u8 == b)
    }
    pub fn from_name(s: &str) -> Option<SetId> {
        // Q1 is an alias for L5: same prime (quantum-hardened λ = 128).
        if s.eq_ignore_ascii_case("Q1") {
            return Some(SetId::L5);
        }
        SetId::ALL
            .iter()
            .copied()
            .find(|x| x.name().eq_ignore_ascii_case(s))
    }
    pub fn name(self) -> &'static str {
        match self {
            SetId::Toy16 => "Toy16",
            SetId::Toy32 => "Toy32",
            SetId::L1 => "L1",
            SetId::L3 => "L3",
            SetId::L5 => "L5",
            SetId::Q3 => "Q3",
            SetId::Q5 => "Q5",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub kind: Kind,
    pub set: SetId,
}

pub fn encode(kind: Kind, set: SetId, payload: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(HEADER_LEN + payload.len());
    v.extend_from_slice(MAGIC);
    v.push(VERSION);
    v.push(kind as u8);
    v.push(set as u8);
    v.push(0);
    v.extend_from_slice(payload);
    v
}

/// Parse a container.  Returns the header and the payload slice, or a
/// description of what is wrong.
pub fn decode(bytes: &[u8]) -> Result<(Header, &[u8]), &'static str> {
    if bytes.len() < HEADER_LEN {
        return Err("file too short for an EigenX header");
    }
    if &bytes[..4] != MAGIC {
        return Err("not an EigenX file (bad magic)");
    }
    if bytes[4] != VERSION {
        return Err("unsupported EigenX format version");
    }
    let kind = Kind::from_u8(bytes[5]).ok_or("unknown object kind")?;
    let set = SetId::from_u8(bytes[6]).ok_or("unknown parameter set")?;
    if bytes[7] != 0 {
        return Err("reserved header byte is not zero");
    }
    Ok((Header { kind, set }, &bytes[HEADER_LEN..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_rejections() {
        let v = encode(Kind::KemCiphertext, SetId::L1, b"abc");
        let (h, p) = decode(&v).unwrap();
        assert_eq!(h.kind, Kind::KemCiphertext);
        assert_eq!(h.set, SetId::L1);
        assert_eq!(p, b"abc");
        assert!(decode(b"EIG").is_err());
        assert!(decode(b"NOPE\x01\x01\x01\x00").is_err());
        assert!(decode(b"EIGX\x02\x01\x01\x00").is_err());
        assert!(decode(b"EIGX\x01\x09\x01\x00").is_err());
        assert!(decode(b"EIGX\x01\x01\x63\x00").is_err());
        assert!(decode(b"EIGX\x01\x01\x06\x00").is_err(), "no set id 6 (Q1)");
        assert!(decode(b"EIGX\x01\x01\x01\x01").is_err());
        for s in SetId::ALL {
            assert_eq!(SetId::from_name(s.name()), Some(s));
            assert_eq!(SetId::from_u8(s as u8), Some(s));
        }
        assert_eq!(SetId::from_name("Q1"), Some(SetId::L5));
        assert_eq!(SetId::from_name("q1"), Some(SetId::L5));
    }
}
