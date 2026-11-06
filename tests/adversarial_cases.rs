use epic_snip_poc::{
    Ledger, LedgerError, PaymentDescriptor, SpendAuthorization,
    test_support::{test_signing_key, transaction_context},
};

#[test]
fn authorization_for_a_does_not_apply_to_b_or_tampered_id() {
    let alice = test_signing_key();
    let bob = test_signing_key();
    let mut ledger = Ledger::new(0);
    let d = PaymentDescriptor::new(bob.verifying_key(), alice.verifying_key());
    let a = ledger.create_payment(&d, 1, 5).unwrap();
    let b = ledger.create_payment(&d, 1, 5).unwrap();
    let auth = SpendAuthorization::claim(ledger.payment(&a).unwrap(), &bob, transaction_context(1));
    let mut replay = auth.clone();
    replay.payment_id = b;
    assert_eq!(ledger.claim(replay), Err(LedgerError::InvalidSignature));
    let mut tampered = auth;
    tampered.payment_id = [99; 32];
    assert!(matches!(
        ledger.claim(tampered),
        Err(LedgerError::UnknownPayment(_))
    ));
}

#[test]
fn duplicate_submission_and_decreasing_height_are_rejected() {
    let alice = test_signing_key();
    let bob = test_signing_key();
    let mut ledger = Ledger::new(2);
    let id = ledger
        .create_payment(
            &PaymentDescriptor::new(bob.verifying_key(), alice.verifying_key()),
            1,
            5,
        )
        .unwrap();
    let payment = ledger.payment(&id).unwrap().clone();
    let auth = SpendAuthorization::claim(&payment, &bob, transaction_context(2));
    ledger.claim(auth.clone()).unwrap();
    assert_eq!(ledger.claim(auth), Err(LedgerError::AlreadySpent));
    assert!(matches!(
        ledger.advance_height(1),
        Err(LedgerError::HeightCannotDecrease { .. })
    ));
}
