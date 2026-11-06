use epic_snip_poc::{
    Ledger, LedgerError, PaymentDescriptor, PaymentStatus, SpendAuthorization,
    test_support::{test_signing_key, transaction_context},
};

#[test]
fn refund_is_rejected_before_and_allowed_at_height() {
    let alice = test_signing_key();
    let bob = test_signing_key();
    let mut ledger = Ledger::new(9);
    let id = ledger
        .create_payment(
            &PaymentDescriptor::new(bob.verifying_key(), alice.verifying_key()),
            7,
            10,
        )
        .unwrap();
    let payment = ledger.payment(&id).unwrap().clone();
    assert!(matches!(
        ledger.refund(SpendAuthorization::refund(
            &payment,
            &alice,
            transaction_context(2)
        )),
        Err(LedgerError::RefundTooEarly { .. })
    ));
    ledger.advance_height(10).unwrap();
    ledger
        .refund(SpendAuthorization::refund(
            &payment,
            &alice,
            transaction_context(3),
        ))
        .unwrap();
    assert_eq!(ledger.payment(&id).unwrap().status, PaymentStatus::Refunded);
}

#[test]
fn bob_can_claim_after_refund_height_if_unspent() {
    let alice = test_signing_key();
    let bob = test_signing_key();
    let mut ledger = Ledger::new(10);
    let id = ledger
        .create_payment(
            &PaymentDescriptor::new(bob.verifying_key(), alice.verifying_key()),
            7,
            10,
        )
        .unwrap();
    let payment = ledger.payment(&id).unwrap().clone();
    ledger
        .claim(SpendAuthorization::claim(
            &payment,
            &bob,
            transaction_context(4),
        ))
        .unwrap();
}
