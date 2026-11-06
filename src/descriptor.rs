use ed25519_dalek::VerifyingKey;
use sha2::{Digest, Sha256};

/// Bobs one-time payment instructions. The refund key belongs to Alice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentDescriptor {
    pub receiver_public_key: VerifyingKey,
    pub refund_public_key: VerifyingKey,
    pub descriptor_id: [u8; 32],
}

impl PaymentDescriptor {
    #[must_use]
    pub fn new(receiver_public_key: VerifyingKey, refund_public_key: VerifyingKey) -> Self {
        let mut hash = Sha256::new();
        hash.update(b"EPIC-SNIP-POC-v0|descriptor|");
        hash.update(receiver_public_key.as_bytes());
        hash.update(refund_public_key.as_bytes());
        Self {
            receiver_public_key,
            refund_public_key,
            descriptor_id: hash.finalize().into(),
        }
    }
}
