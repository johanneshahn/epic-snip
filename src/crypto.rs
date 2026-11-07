//! Cryptographic primitives aligned with the locally checked-out Epic stack.
//!
//! This module uses the vendored `grin_secp256k1zkp` implementation found in
//! the local Epic source checkout. It deliberately contains no transaction,
//! range-proof, kernel, networking, wallet, or consensus logic.

use secp256k1zkp::{
    ContextFlag, Message, Secp256k1, Signature, aggsig,
    key::{PublicKey, SecretKey},
    pedersen::Commitment,
};
use sha2::{Digest, Sha256};

use crate::PaymentId;

/// A local wrapper around the Pedersen and single-signer `AgSig` facilities.
///
/// Its `Commit` context is the capability mode used by the Epic bindings for
/// commitments and supports signing and verification as well.
#[derive(Debug)]
pub struct EpicCrypto {
    secp: Secp256k1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CryptoError {
    BackendFailure,
}

/// Canonical public data bound into an `AgSig` spend authorization.
#[derive(Clone, Copy, Debug)]
pub struct SpendTranscript {
    pub payment_id: PaymentId,
    pub commitment: Commitment,
}

impl Default for EpicCrypto {
    fn default() -> Self {
        Self::new()
    }
}

impl EpicCrypto {
    #[must_use]
    pub fn new() -> Self {
        Self {
            secp: Secp256k1::with_caps(ContextFlag::Commit),
        }
    }

    /// Makes a Pedersen commitment `value * H + blind * G`.
    ///
    /// # Errors
    /// Returns [`CryptoError::BackendFailure`] if the underlying binding rejects
    /// the scalar or cannot serialize the commitment.
    pub fn commit(&self, value: u64, blind: SecretKey) -> Result<Commitment, CryptoError> {
        self.secp
            .commit(value, blind)
            .map_err(|_| CryptoError::BackendFailure)
    }

    /// Adds positive and subtracts negative Pedersen commitments.
    ///
    /// # Errors
    /// Returns [`CryptoError::BackendFailure`] for malformed commitments or an
    /// invalid resulting commitment.
    pub fn commitment_sum(
        &self,
        positive: Vec<Commitment>,
        negative: Vec<Commitment>,
    ) -> Result<Commitment, CryptoError> {
        self.secp
            .commit_sum(positive, negative)
            .map_err(|_| CryptoError::BackendFailure)
    }

    /// Produces an Epic `AgSig` single-signer signature bound to a transcript.
    ///
    /// The signature transcript includes the payment identity and the actual
    /// Pedersen commitment.
    ///
    /// # Errors
    /// Returns [`CryptoError::BackendFailure`] if signing fails.
    pub fn sign_spend(
        &self,
        transcript: &SpendTranscript,
        key: &SecretKey,
    ) -> Result<Signature, CryptoError> {
        let message = spend_message(transcript)?;
        aggsig::sign_single(&self.secp, &message, key, None, None, None, None, None)
            .map_err(|_| CryptoError::BackendFailure)
    }

    #[must_use]
    pub fn verify_spend(
        &self,
        transcript: &SpendTranscript,
        signature: &Signature,
        public_key: &PublicKey,
    ) -> bool {
        let Ok(message) = spend_message(transcript) else {
            return false;
        };
        aggsig::verify_single(
            &self.secp, signature, &message, None, public_key, None, None, false,
        )
    }

    /// Derives the public key for a private scalar.
    ///
    /// # Errors
    /// Returns [`CryptoError::BackendFailure`] when the binding rejects the key.
    pub fn public_key(&self, key: &SecretKey) -> Result<PublicKey, CryptoError> {
        PublicKey::from_secret_key(&self.secp, key).map_err(|_| CryptoError::BackendFailure)
    }

    #[must_use]
    pub fn is_balanced(&self, positive: Vec<Commitment>, negative: Vec<Commitment>) -> bool {
        self.secp.verify_commit_sum(positive, negative)
    }
}

fn spend_message(transcript: &SpendTranscript) -> Result<Message, CryptoError> {
    let mut hash = Sha256::new();
    hash.update(b"EPIC-SNIP-POC-v1|aggsig-spend|");
    hash.update(transcript.payment_id);
    hash.update(transcript.commitment.as_ref());
    Message::from_slice(&hash.finalize()).map_err(|_| CryptoError::BackendFailure)
}
