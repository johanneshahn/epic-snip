use std::collections::HashMap;

use ed25519_dalek::Verifier;
use sha2::{Digest, Sha256};

use crate::{
    Amount, LedgerError, PaymentDescriptor, PaymentId, PaymentStatus, PendingPayment, SpendPath,
    authorization::SpendAuthorization,
};

/// The sole owner of payment state. State transitions happen only here.
#[derive(Debug, Default)]
pub struct Ledger {
    current_height: u64,
    payments: HashMap<PaymentId, PendingPayment>,
    next_nonce: u64,
}

impl Ledger {
    #[must_use]
    pub fn new(initial_height: u64) -> Self {
        Self {
            current_height: initial_height,
            ..Self::default()
        }
    }

    #[must_use]
    pub const fn current_height(&self) -> u64 {
        self.current_height
    }

    /// Advances the simulated chain height.
    ///
    /// # Errors
    /// Returns [`LedgerError::HeightCannotDecrease`] if `new_height` is lower.
    pub fn advance_height(&mut self, new_height: u64) -> Result<(), LedgerError> {
        if new_height < self.current_height {
            return Err(LedgerError::HeightCannotDecrease {
                current_height: self.current_height,
                requested_height: new_height,
            });
        }
        self.current_height = new_height;
        Ok(())
    }

    /// Creates a pending payment from a descriptor.
    ///
    /// # Errors
    /// Returns an error if the test nonce is exhausted or an identifier collides.
    pub fn create_payment(
        &mut self,
        descriptor: &PaymentDescriptor,
        amount: Amount,
        refund_height: u64,
    ) -> Result<PaymentId, LedgerError> {
        let nonce = self.next_nonce;
        self.next_nonce = self
            .next_nonce
            .checked_add(1)
            .ok_or(LedgerError::NonceExhausted)?;
        let commitment_placeholder = commitment(descriptor, amount, refund_height, nonce);
        let payment_id = payment_id(
            descriptor,
            amount,
            refund_height,
            nonce,
            &commitment_placeholder,
        );
        let payment = PendingPayment {
            payment_id,
            amount,
            receiver_public_key: descriptor.receiver_public_key,
            refund_public_key: descriptor.refund_public_key,
            refund_height,
            status: PaymentStatus::Pending,
            commitment_placeholder,
        };
        if self.payments.insert(payment_id, payment).is_some() {
            return Err(LedgerError::DuplicatePayment(payment_id));
        }
        Ok(payment_id)
    }

    #[must_use]
    pub fn payment(&self, payment_id: &PaymentId) -> Option<&PendingPayment> {
        self.payments.get(payment_id)
    }

    /// Validates and records Bobs claim authorization.
    ///
    /// # Errors
    /// Returns an error for an unknown, spent, or incorrectly authorized payment.
    pub fn claim(&mut self, authorization: SpendAuthorization) -> Result<(), LedgerError> {
        self.spend(authorization, SpendPath::Claim)
    }

    /// Validates and records Alices refund authorization.
    ///
    /// # Errors
    /// Returns an error for an early, unknown, spent, or incorrectly authorized payment.
    pub fn refund(&mut self, authorization: SpendAuthorization) -> Result<(), LedgerError> {
        self.spend(authorization, SpendPath::Refund)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn spend(
        &mut self,
        authorization: SpendAuthorization,
        expected_path: SpendPath,
    ) -> Result<(), LedgerError> {
        let current_height = self.current_height;
        let payment = self
            .payments
            .get_mut(&authorization.payment_id)
            .ok_or(LedgerError::UnknownPayment(authorization.payment_id))?;
        if authorization.path != expected_path {
            return Err(LedgerError::WrongPath {
                expected: expected_path,
                actual: authorization.path,
            });
        }
        if payment.status != PaymentStatus::Pending {
            return Err(LedgerError::AlreadySpent);
        }
        if payment.payment_id != authorization.payment_id {
            return Err(LedgerError::InvalidPaymentBinding);
        }
        if expected_path == SpendPath::Refund && current_height < payment.refund_height {
            return Err(LedgerError::RefundTooEarly {
                current_height,
                refund_height: payment.refund_height,
            });
        }
        let key = match expected_path {
            SpendPath::Claim => &payment.receiver_public_key,
            SpendPath::Refund => &payment.refund_public_key,
        };
        key.verify(
            &SpendAuthorization::message(
                payment,
                authorization.path,
                authorization.transaction_context,
            ),
            &authorization.signature,
        )
        .map_err(|_| LedgerError::InvalidSignature)?;
        payment.status = match expected_path {
            SpendPath::Claim => PaymentStatus::Claimed,
            SpendPath::Refund => PaymentStatus::Refunded,
        };
        Ok(())
    }
}

fn commitment(
    descriptor: &PaymentDescriptor,
    amount: Amount,
    refund_height: u64,
    nonce: u64,
) -> [u8; 32] {
    hash(&[
        b"EPIC-SNIP-POC-v0|NON-CONSENSUS-commitment-placeholder|",
        &descriptor.descriptor_id,
        &amount.to_le_bytes(),
        &refund_height.to_le_bytes(),
        &nonce.to_le_bytes(),
    ])
}

fn payment_id(
    descriptor: &PaymentDescriptor,
    amount: Amount,
    refund_height: u64,
    nonce: u64,
    placeholder: &[u8; 32],
) -> PaymentId {
    hash(&[
        b"EPIC-SNIP-POC-v0|payment|",
        &descriptor.descriptor_id,
        descriptor.receiver_public_key.as_bytes(),
        descriptor.refund_public_key.as_bytes(),
        &amount.to_le_bytes(),
        &refund_height.to_le_bytes(),
        &nonce.to_le_bytes(),
        placeholder,
    ])
}

fn hash(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().into()
}
