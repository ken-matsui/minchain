extern crate crypto_hash;

use self::crypto_hash::{hex_digest, Algorithm};

pub fn get_double_sha256(s: String) -> String {
    let digest = hex_digest(Algorithm::SHA256, s.as_bytes());
    hex_digest(Algorithm::SHA256, digest.as_bytes())
}
