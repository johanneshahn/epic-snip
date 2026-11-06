use epic_snip_poc::{OneSidedError, OneSidedLedger, OneSidedPayment, OneSidedStatus, ReceiverKey};
use secp256k1zkp::{ContextFlag, Secp256k1, key::SecretKey};

fn key() -> SecretKey {
    SecretKey::new(&Secp256k1::with_caps(ContextFlag::Commit), &mut rand::rng())
}

#[test]
fn bob_detects_and_claims_a_one_sided_pending_payment() {
    let bob = ReceiverKey::random();
    let payment = OneSidedPayment::create(42, bob.public_key()).unwrap();
    let capability = bob
        .scan(&payment.envelope)
        .expect("Bob recognizes his envelope");
    let mut ledger = OneSidedLedger::default();
    ledger.register(&payment).unwrap();
    let claimed = ledger.claim(&capability, key()).unwrap();
    assert_ne!(claimed.commitment, payment.envelope.commitment);
    assert_eq!(
        ledger.status(payment.envelope.payment_id),
        Some(OneSidedStatus::Claimed)
    );
}

#[test]
fn a_different_receiver_cannot_recognize_the_payment() {
    let bob = ReceiverKey::random();
    let eve = ReceiverKey::random();
    let payment = OneSidedPayment::create(42, bob.public_key()).unwrap();
    assert_eq!(
        eve.scan(&payment.envelope).unwrap_err(),
        OneSidedError::InvalidCommitment
    );
}

#[test]
fn alice_can_win_the_race_before_bob_claims() {
    let bob = ReceiverKey::random();
    let payment = OneSidedPayment::create(42, bob.public_key()).unwrap();
    let bob_capability = bob.scan(&payment.envelope).unwrap();
    let mut ledger = OneSidedLedger::default();
    ledger.register(&payment).unwrap();
    ledger
        .sender_spend(&payment.sender_capability, key())
        .unwrap();
    assert_eq!(
        ledger.status(payment.envelope.payment_id),
        Some(OneSidedStatus::SenderSpent)
    );
    assert_eq!(
        ledger.claim(&bob_capability, key()).unwrap_err(),
        OneSidedError::AlreadySpent
    );
}
