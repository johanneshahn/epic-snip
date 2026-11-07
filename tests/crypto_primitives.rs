use epic_snip_poc::{PaymentId, SpendTranscript, crypto::EpicCrypto};
use secp256k1zkp::key::SecretKey;

fn key() -> SecretKey {
    SecretKey::new(
        &secp256k1zkp::Secp256k1::with_caps(secp256k1zkp::ContextFlag::Commit),
        &mut rand::rng(),
    )
}

#[test]
fn pedersen_commitments_have_the_expected_additive_relation() {
    let crypto = EpicCrypto::new();
    let blind_a = key();
    let blind_b = key();
    let blind_total = secp256k1zkp::Secp256k1::with_caps(secp256k1zkp::ContextFlag::Commit)
        .blind_sum(vec![blind_a.clone(), blind_b.clone()], vec![])
        .unwrap();
    let a = crypto.commit(17, blind_a).unwrap();
    let b = crypto.commit(25, blind_b).unwrap();
    let total = crypto.commit(42, blind_total).unwrap();
    assert_eq!(crypto.commitment_sum(vec![a, b], vec![]).unwrap(), total);
    assert!(crypto.is_balanced(vec![a, b], vec![total]));
}

#[test]
fn aggsig_spend_authorization_is_bound_to_payment_id_and_commitment() {
    let crypto = EpicCrypto::new();
    let spending_key = key();
    let public_key = crypto.public_key(&spending_key).unwrap();
    let commitment = crypto.commit(9, key()).unwrap();
    let payment_id: PaymentId = [7; 32];
    let transcript = SpendTranscript {
        payment_id,
        commitment,
    };
    let signature = crypto.sign_spend(&transcript, &spending_key).unwrap();

    assert!(crypto.verify_spend(&transcript, &signature, &public_key));
    // A different payment id must not verify.
    assert!(!crypto.verify_spend(
        &SpendTranscript {
            payment_id: [8; 32],
            commitment,
        },
        &signature,
        &public_key
    ));
    // A different commitment must not verify.
    let other_commitment = crypto.commit(9, key()).unwrap();
    assert!(!crypto.verify_spend(
        &SpendTranscript {
            payment_id,
            commitment: other_commitment,
        },
        &signature,
        &public_key
    ));
}
