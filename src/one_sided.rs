//! A research-only one-sided pending-payment model.
//!
//! This is **not** an Epic Cash transaction format, range-proof extension, or
//! consensus proposal. In particular, its `PendingEnvelope` is a wallet-level
//! notification object. Epic's current output and range-proof formats do not
//! carry this object's ephemeral public key.

use std::collections::HashMap;

use secp256k1zkp::{
    ContextFlag, Message, Secp256k1, aggsig,
    ecdh::SharedSecret,
    key::{PublicKey, SecretKey},
    pedersen::Commitment,
};
use sha2::{Digest, Sha256};

use crate::{Amount, PaymentId};

const DOMAIN: &[u8] = b"EPIC-SNIP-POC-v2|one-sided-pending|";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OneSidedError {
    InvalidDerivedKey,
    InvalidCommitment,
    UnknownPayment,
    AlreadySpent,
    InvalidAuthorization,
}

/// Bob's long-lived *public* scan descriptor. Publishing it alone does not
/// make a current Epic wallet able to discover outputs: this POC relies on a
/// separate `PendingEnvelope` delivery channel.
#[derive(Clone, Debug)]
pub struct ReceiverKey {
    secret: SecretKey,
    public: PublicKey,
}

impl ReceiverKey {
    #[must_use]
    pub fn random() -> Self {
        let secp = Secp256k1::with_caps(ContextFlag::Commit);
        let secret = SecretKey::new(&secp, &mut rand::rng());
        let public = PublicKey::from_secret_key(&secp, &secret).expect("valid secret key");
        Self { secret, public }
    }

    #[must_use]
    pub fn public_key(&self) -> PublicKey {
        self.public.clone()
    }

    /// Attempts to recognize an envelope and, only on a match, returns the
    /// shared capability needed to authorize a claim.
    pub fn scan(&self, envelope: &PendingEnvelope) -> Result<ClaimCapability, OneSidedError> {
        let secp = Secp256k1::with_caps(ContextFlag::Commit);
        let blind = derive_shared_blind(
            &secp,
            &self.secret,
            &envelope.ephemeral_public_key,
            &envelope.ephemeral_public_key,
            &envelope.receiver_public_key,
            envelope.amount,
        )?;
        let expected = secp
            .commit(envelope.amount, blind.clone())
            .map_err(|_| OneSidedError::InvalidCommitment)?;
        if expected != envelope.commitment {
            return Err(OneSidedError::InvalidCommitment);
        }
        let expected_public = PublicKey::from_secret_key(&secp, &blind)
            .map_err(|_| OneSidedError::InvalidDerivedKey)?;
        if expected_public != envelope.shared_spend_public_key {
            return Err(OneSidedError::InvalidCommitment);
        }
        Ok(ClaimCapability {
            payment_id: envelope.payment_id,
            shared_spend_key: blind,
        })
    }
}

/// The public, one-way wallet notification needed by this POC. It is not
/// placed in an Epic output, slate, kernel, or existing range-proof message.
#[derive(Clone, Debug)]
pub struct PendingEnvelope {
    pub payment_id: PaymentId,
    pub amount: Amount,
    pub commitment: Commitment,
    pub ephemeral_public_key: PublicKey,
    pub receiver_public_key: PublicKey,
    pub shared_spend_public_key: PublicKey,
}

/// Alice keeps this capability, so she can race Bob's later claim. That is an
/// intentional limitation of the non-interactive construction.
#[derive(Clone, Debug)]
pub struct SenderCapability {
    payment_id: PaymentId,
    shared_spend_key: SecretKey,
}

#[derive(Clone, Debug)]
pub struct ClaimCapability {
    payment_id: PaymentId,
    shared_spend_key: SecretKey,
}

#[derive(Clone, Debug)]
pub struct OneSidedPayment {
    pub envelope: PendingEnvelope,
    pub sender_capability: SenderCapability,
}

#[derive(Clone, Debug)]
pub struct ClaimedOutput {
    pub commitment: Commitment,
    pub ownership_public_key: PublicKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OneSidedStatus {
    Pending,
    Claimed,
    SenderSpent,
}

#[derive(Clone, Copy, Debug)]
enum Path {
    Claim,
    SenderSpend,
}

impl Path {
    fn tag(self) -> &'static [u8] {
        match self {
            Self::Claim => b"claim",
            Self::SenderSpend => b"sender-spend",
        }
    }
}

/// An in-memory race model. It checks a shared-key AgSig authorization but
/// does not create a valid Mimblewimble transaction or enforce a kernel sum.
#[derive(Default)]
pub struct OneSidedLedger {
    payments: HashMap<PaymentId, (PendingEnvelope, OneSidedStatus)>,
}

impl OneSidedLedger {
    pub fn register(&mut self, payment: &OneSidedPayment) -> Result<(), OneSidedError> {
        if self.payments.contains_key(&payment.envelope.payment_id) {
            return Err(OneSidedError::AlreadySpent);
        }
        self.payments.insert(
            payment.envelope.payment_id,
            (payment.envelope.clone(), OneSidedStatus::Pending),
        );
        Ok(())
    }

    #[must_use]
    pub fn status(&self, id: PaymentId) -> Option<OneSidedStatus> {
        self.payments.get(&id).map(|(_, s)| *s)
    }

    pub fn claim(
        &mut self,
        capability: &ClaimCapability,
        owner_key: SecretKey,
    ) -> Result<ClaimedOutput, OneSidedError> {
        self.spend(
            capability.payment_id,
            &capability.shared_spend_key,
            owner_key,
            Path::Claim,
        )
    }

    pub fn sender_spend(
        &mut self,
        capability: &SenderCapability,
        owner_key: SecretKey,
    ) -> Result<ClaimedOutput, OneSidedError> {
        self.spend(
            capability.payment_id,
            &capability.shared_spend_key,
            owner_key,
            Path::SenderSpend,
        )
    }

    fn spend(
        &mut self,
        id: PaymentId,
        shared_key: &SecretKey,
        owner_key: SecretKey,
        path: Path,
    ) -> Result<ClaimedOutput, OneSidedError> {
        let (envelope, status) = self
            .payments
            .get(&id)
            .ok_or(OneSidedError::UnknownPayment)?;
        if *status != OneSidedStatus::Pending {
            return Err(OneSidedError::AlreadySpent);
        }
        let secp = Secp256k1::with_caps(ContextFlag::Commit);
        let destination = secp
            .commit(envelope.amount, owner_key.clone())
            .map_err(|_| OneSidedError::InvalidCommitment)?;
        let message = authorization_message(envelope, &destination, path)?;
        let signature =
            aggsig::sign_single(&secp, &message, shared_key, None, None, None, None, None)
                .map_err(|_| OneSidedError::InvalidAuthorization)?;
        if !aggsig::verify_single(
            &secp,
            &signature,
            &message,
            None,
            &envelope.shared_spend_public_key,
            None,
            None,
            false,
        ) {
            return Err(OneSidedError::InvalidAuthorization);
        }
        let output = ClaimedOutput {
            commitment: destination,
            ownership_public_key: PublicKey::from_secret_key(&secp, &owner_key)
                .map_err(|_| OneSidedError::InvalidDerivedKey)?,
        };
        self.payments.get_mut(&id).expect("checked above").1 = match path {
            Path::Claim => OneSidedStatus::Claimed,
            Path::SenderSpend => OneSidedStatus::SenderSpent,
        };
        Ok(output)
    }
}

impl OneSidedPayment {
    /// Alice creates a new ECDH-derived, shared pending output for Bob's
    /// published scan key. Alice receives no response from Bob.
    pub fn create(amount: Amount, receiver_public_key: PublicKey) -> Result<Self, OneSidedError> {
        let secp = Secp256k1::with_caps(ContextFlag::Commit);
        let ephemeral_secret = SecretKey::new(&secp, &mut rand::rng());
        let ephemeral_public_key = PublicKey::from_secret_key(&secp, &ephemeral_secret)
            .map_err(|_| OneSidedError::InvalidDerivedKey)?;
        let shared_spend_key = derive_shared_blind(
            &secp,
            &ephemeral_secret,
            &receiver_public_key,
            &ephemeral_public_key,
            &receiver_public_key,
            amount,
        )?;
        let commitment = secp
            .commit(amount, shared_spend_key.clone())
            .map_err(|_| OneSidedError::InvalidCommitment)?;
        let shared_spend_public_key = PublicKey::from_secret_key(&secp, &shared_spend_key)
            .map_err(|_| OneSidedError::InvalidDerivedKey)?;
        let payment_id = payment_id(&commitment, &ephemeral_public_key, &receiver_public_key);
        Ok(Self {
            envelope: PendingEnvelope {
                payment_id,
                amount,
                commitment,
                ephemeral_public_key,
                receiver_public_key,
                shared_spend_public_key,
            },
            sender_capability: SenderCapability {
                payment_id,
                shared_spend_key,
            },
        })
    }
}

fn derive_shared_blind(
    secp: &Secp256k1,
    secret: &SecretKey,
    peer: &PublicKey,
    ephemeral: &PublicKey,
    receiver: &PublicKey,
    amount: Amount,
) -> Result<SecretKey, OneSidedError> {
    let shared = SharedSecret::new(secp, peer, secret);
    for counter in 0_u8..=255 {
        let mut hash = Sha256::new();
        hash.update(DOMAIN);
        hash.update(b"blind|");
        hash.update(&shared[..]);
        hash.update(ephemeral.serialize_vec(secp, true).as_slice());
        hash.update(receiver.serialize_vec(secp, true).as_slice());
        hash.update(amount.to_le_bytes());
        hash.update([counter]);
        if let Ok(key) = SecretKey::from_slice(secp, &hash.finalize()) {
            return Ok(key);
        }
    }
    Err(OneSidedError::InvalidDerivedKey)
}

fn payment_id(commitment: &Commitment, ephemeral: &PublicKey, receiver: &PublicKey) -> PaymentId {
    let secp = Secp256k1::with_caps(ContextFlag::Commit);
    let mut hash = Sha256::new();
    hash.update(DOMAIN);
    hash.update(b"id|");
    hash.update(commitment.as_ref());
    hash.update(ephemeral.serialize_vec(&secp, true).as_slice());
    hash.update(receiver.serialize_vec(&secp, true).as_slice());
    hash.finalize().into()
}

fn authorization_message(
    envelope: &PendingEnvelope,
    destination: &Commitment,
    path: Path,
) -> Result<Message, OneSidedError> {
    let mut hash = Sha256::new();
    hash.update(DOMAIN);
    hash.update(b"authorization|");
    hash.update(envelope.payment_id);
    hash.update(path.tag());
    hash.update(envelope.commitment.as_ref());
    hash.update(destination.as_ref());
    hash.update(envelope.amount.to_le_bytes());
    Message::from_slice(&hash.finalize()).map_err(|_| OneSidedError::InvalidAuthorization)
}
