use ed25519_dalek::VerifyingKey;

use crate::{Amount, PaymentId, PaymentStatus};

/// An in-memory payment only. `commitment_placeholder` is a deterministic hash,
/// not a Pedersen commitment and provides no value-hiding property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingPayment {
    pub payment_id: PaymentId,
    pub amount: Amount,
    pub receiver_public_key: VerifyingKey,
    pub refund_public_key: VerifyingKey,
    pub refund_height: u64,
    pub status: PaymentStatus,
    pub commitment_placeholder: [u8; 32],
}
