use epic_snip_poc::{
    Ledger, LedgerError, PaymentDescriptor, SpendAuthorization,
    test_support::{test_signing_key, transaction_context},
};

#[test]
fn foreign_key_cannot_claim_or_refund() {
    let alice = test_signing_key();
    let bob = test_signing_key();
    let attacker = test_signing_key();
    let mut ledger = Ledger::new(10);
    let id = ledger
        .create_payment(
            &PaymentDescriptor::new(bob.verifying_key(), alice.verifying_key()),
            1,
            10,
        )
        .unwrap();
    let payment = ledger.payment(&id).unwrap().clone();
    assert_eq!(
        ledger.claim(SpendAuthorization::claim(
            &payment,
            &attacker,
            transaction_context(1)
        )),
        Err(LedgerError::InvalidSignature)
    );
    assert_eq!(
        ledger.refund(SpendAuthorization::refund(
            &payment,
            &attacker,
            transaction_context(2)
        )),
        Err(LedgerError::InvalidSignature)
    );
}

#[test]
fn claim_signature_cannot_be_used_for_refund() {
    let alice = test_signing_key();
    let bob = test_signing_key();
    let mut ledger = Ledger::new(10);
    let id = ledger
        .create_payment(
            &PaymentDescriptor::new(bob.verifying_key(), alice.verifying_key()),
            1,
            10,
        )
        .unwrap();
    let payment = ledger.payment(&id).unwrap().clone();
    let mut auth = SpendAuthorization::claim(&payment, &bob, transaction_context(3));
    auth.path = epic_snip_poc::SpendPath::Refund;
    assert_eq!(ledger.refund(auth), Err(LedgerError::InvalidSignature));
}
