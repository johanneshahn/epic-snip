use epic_snip_poc::{
    Ledger, PaymentDescriptor, PaymentStatus, SpendAuthorization,
    test_support::{test_signing_key, transaction_context},
};

#[test]
fn bob_can_claim_a_valid_payment() {
    let alice = test_signing_key();
    let bob = test_signing_key();
    let mut ledger = Ledger::new(4);
    let id = ledger
        .create_payment(
            &PaymentDescriptor::new(bob.verifying_key(), alice.verifying_key()),
            42,
            10,
        )
        .unwrap();
    let payment = ledger.payment(&id).unwrap().clone();
    ledger
        .claim(SpendAuthorization::claim(
            &payment,
            &bob,
            transaction_context(1),
        ))
        .unwrap();
    assert_eq!(ledger.payment(&id).unwrap().status, PaymentStatus::Claimed);
}
