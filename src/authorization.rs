use ed25519_dalek::{Signature, Signer, SigningKey};

use crate::{PaymentId, PendingPayment, SpendPath, TransactionContext};

#[derive(Clone, Debug)]
pub struct SpendAuthorization {
    pub payment_id: PaymentId,
    pub path: SpendPath,
    pub transaction_context: TransactionContext,
    pub signature: Signature,
}

impl SpendAuthorization {
    #[must_use]
    pub fn claim(
        payment: &PendingPayment,
        key: &SigningKey,
        transaction_context: TransactionContext,
    ) -> Self {
        Self::sign(payment, SpendPath::Claim, key, transaction_context)
    }

    #[must_use]
    pub fn refund(
        payment: &PendingPayment,
        key: &SigningKey,
        transaction_context: TransactionContext,
    ) -> Self {
        Self::sign(payment, SpendPath::Refund, key, transaction_context)
    }

    fn sign(
        payment: &PendingPayment,
        path: SpendPath,
        key: &SigningKey,
        transaction_context: TransactionContext,
    ) -> Self {
        let signature = key.sign(&Self::message(payment, path, transaction_context));
        Self {
            payment_id: payment.payment_id,
            path,
            transaction_context,
            signature,
        }
    }

    pub(crate) fn message(
        payment: &PendingPayment,
        path: SpendPath,
        transaction_context: TransactionContext,
    ) -> Vec<u8> {
        let mut message = Vec::with_capacity(32 * 4 + 8 + 32);
        message.extend_from_slice(b"EPIC-SNIP-POC-v0|spend|");
        message.extend_from_slice(&payment.payment_id);
        message.extend_from_slice(path.tag());
        message.extend_from_slice(&payment.commitment_placeholder);
        message.extend_from_slice(&payment.amount.to_le_bytes());
        message.extend_from_slice(&payment.refund_height.to_le_bytes());
        message.extend_from_slice(&transaction_context);
        message
    }
}
