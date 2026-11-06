use epic_snip_poc::{
    Ledger, LedgerError, PaymentDescriptor, SpendAuthorization,
    test_support::{test_signing_key, transaction_context},
};
use std::sync::{Arc, Mutex};
use std::thread;

fn fixture() -> (
    Ledger,
    epic_snip_poc::PendingPayment,
    ed25519_dalek::SigningKey,
    ed25519_dalek::SigningKey,
) {
    let alice = test_signing_key();
    let bob = test_signing_key();
    let mut ledger = Ledger::new(20);
    let id = ledger
        .create_payment(
            &PaymentDescriptor::new(bob.verifying_key(), alice.verifying_key()),
            1,
            20,
        )
        .unwrap();
    let payment = ledger.payment(&id).unwrap().clone();
    (ledger, payment, alice, bob)
}

#[test]
fn one_path_makes_the_other_invalid() {
    let (mut ledger, payment, alice, bob) = fixture();
    ledger
        .claim(SpendAuthorization::claim(
            &payment,
            &bob,
            transaction_context(1),
        ))
        .unwrap();
    assert_eq!(
        ledger.refund(SpendAuthorization::refund(
            &payment,
            &alice,
            transaction_context(2)
        )),
        Err(LedgerError::AlreadySpent)
    );
}

#[test]
fn two_concurrent_spends_have_exactly_one_winner() {
    let (ledger, payment, alice, bob) = fixture();
    let shared = Arc::new(Mutex::new(ledger));
    let claim = SpendAuthorization::claim(&payment, &bob, transaction_context(3));
    let refund = SpendAuthorization::refund(&payment, &alice, transaction_context(4));
    let a = Arc::clone(&shared);
    let b = Arc::clone(&shared);
    let first = thread::spawn(move || a.lock().unwrap().claim(claim).is_ok());
    let second = thread::spawn(move || b.lock().unwrap().refund(refund).is_ok());
    assert_eq!(
        u8::from(first.join().unwrap()) + u8::from(second.join().unwrap()),
        1
    );
}
