//! Deterministic random bit generator for known-answer tests and
//! reproducible experiments: the output stream is SHAKE256(domain || seed).
//!
//! It implements `CryptoRng` only so that it can be passed to the KEM's
//! generic APIs.  It is **not** a secure RNG for production use, because
//! anyone who knows the seed knows every output.  Use `rand_core::OsRng`
//! for real keys.

use rand_core::{CryptoRng, RngCore};
use sha3::digest::{ExtendableOutput, Update, XofReader};
use sha3::{Shake256, Shake256Reader};

pub struct Drbg {
    reader: Shake256Reader,
}

impl Drbg {
    pub fn new(seed: &[u8]) -> Self {
        let mut h = Shake256::default();
        h.update(b"EigenSeta-DRBG");
        h.update(&(seed.len() as u64).to_le_bytes());
        h.update(seed);
        Drbg {
            reader: h.finalize_xof(),
        }
    }
    /// A DRBG for test vector number `index` of a named suite.
    pub fn for_vector(suite: &str, index: u64) -> Self {
        let mut seed = Vec::with_capacity(suite.len() + 8);
        seed.extend_from_slice(suite.as_bytes());
        seed.extend_from_slice(&index.to_le_bytes());
        Self::new(&seed)
    }
}

impl RngCore for Drbg {
    fn next_u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        self.reader.read(&mut b);
        u32::from_le_bytes(b)
    }
    fn next_u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        self.reader.read(&mut b);
        u64::from_le_bytes(b)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.reader.read(dest);
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for Drbg {}
