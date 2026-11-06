//! Test conveniences. Never use these routines as wallet key management.
use ed25519_dalek::SigningKey;
use rand_core::OsRng;

pub fn test_signing_key() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

#[must_use]
pub const fn transaction_context(value: u8) -> [u8; 32] {
    [value; 32]
}
