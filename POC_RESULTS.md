# Epic SNIP — Proof of Concept Results

**Status:** Technically validated Proof of Concept (Usernet only).
**Date:** 2026-11-08
**Scope:** Semi-non-interactive payments (SNIP) on Epic Cash, Usernet research
prototype. No Mainnet/Floonet support, no production activation.

---

## 1. Original technical question

Can Epic Cash support semi-non-interactive payments where Bob publishes a single
reusable descriptor, Alice creates multiple independent payments offline, Bob
discovers them by scanning the chain, and Bob claims them without Alice being
online — while preventing Bob from bypassing mandatory claim evidence and
allowing Alice to reclaim unclaimed payments?

Answer: **Yes, through targeted extensions to Epic Cash's existing cryptography
and consensus, not through Epic Cash as-is.**

---

## 2. What was extended in Epic Cash

### Cryptography (epic_core)
- `SharedReceiptKeyHashes`: added `alice` hash (32 bytes) alongside `bob`.
- `ReceiptPath`: added `AliceSpend` tag.
- `ReceiptSpendAuthorization`: new struct binding Alice's signature to the
  Receipt, kernel excess, fee, kernel hash, and chain id.
- `build_alice_spend_transaction`: new builder for the Alice reclaim path.
- Domain-separated key derivation: receipt tweak, shared spend blind, Alice
  authorization key, all distinct domains.

### Consensus (epic_core/transaction.rs)
- Two-branch `verify_receipt_resolution`: BobClaim (signed by b_i) OR
  AliceSpend (Plain kernel + authorization signed by a_i).
- `verify_receipt_spend_authorizations`: full validation of Alice signatures.
- Receipt-aware cut-through: receipt inputs/outputs exempt from cut-through.
- Activation gate: `verify_receipt_feature_activation` (test-flag gated for
  Usernet; height-based activation deferred).

### Wallet (epic_snip_wallet)
- Deterministic key derivation from Epic seed (hardened paths, domain
  separation).
- Duplicate-safe, restart-safe JSON payment record store.
- Incoming/outgoing receipt scanning and recovery.
- Bob claim and Alice reclaim transaction builders.
- Single-writer file lock (F-04 hardening).

---

## 3. Cryptographic properties demonstrated

| Property | Evidence |
|---|---|
| Alice cannot derive Bob's exclusive claim key `b_i` | `b_i` is derived from Bob's private scan secret + ECDH shared secret. Alice lacks Bob's scan secret; the ECDH shared secret alone does not reveal `b_i` (the tweak is added to Bob's private spend key). |
| Bob cannot derive Alice's exclusive reclaim key `a_i` | `a_i` is derived from Alice's seed via a separate hardened path (`ROLE_ALICE`, `KEY_ALICE_AUTH`). Bob has no access to Alice's seed. The shared blind `q_i` is independent of `a_i`. |
| Independent payments do not share authorization keys | `three_payments_generate_distinct_alice_authorization_keys` (core test). |
| Alice's authorization key is independent of shared blind and Bob's receipt key | `alice_authorization_key_is_not_the_shared_blind_or_bob_receipt_key` (core test). |
| Domain separation prevents key aliasing | `keys_are_domain_separated_and_unique`, `distinct_paths_are_used_for_each_key_type` (wallet unit tests). |
| Authorization signatures are bound to Receipt, kernel, fee, and chain | `wrong_commitment_fails`, `wrong_excess_fails`, `wrong_fee_fails`, `wrong_kernel_hash_fails`, `wrong_chain_identifier_fails` (core tests). |
| Malformed public keys rejected | `PublicKey::from_secret_key` validation in `from_secret_key`. |

### Mathematical reasoning (compact)

Given the receipt output committing to `H(B_i)` and `H(A_i)`:
- `B_i = B + tweak*G` where `tweak = KDF(ECDH(e, S) || ...)`. Computing `b_i = b + tweak` requires Bob's private `b` (scan spend secret). Alice knows `tweak` (she chose `e`) and `B_i` (public) but cannot recover `b` from `B = bG` (discrete log). Thus Alice cannot forge a BobClaim.
- `A_i = a_i G` where `a_i = KDF(Alice_seed || index)`. Bob knows `A_i` (public) but cannot recover `a_i` (discrete log) and has no access to Alice's seed. Thus Bob cannot forge an AliceSpend authorization.
- The shared blind `q_i` is a balance-construction capability, not an authorization. Both parties know `q_i`, but neither can produce the other's branch signature.

---

## 4. Test coverage by verification area

### Area 1: Complete flow
- `phase_a_alice_funds_three_payments` — Alice creates 3 payments, all confirmed.
- `phase_b_bob_discovers_three_payments_by_scan` — Bob discovers all 3 by scanning.
- `phase_c_claims_reclaim_and_restart` — Bob claims R1, Alice reclaims R2, Bob restarts and claims R3.
- `phase_d_conflicts_rejected_after_all_spent` — all conflicting spends rejected.
- `phase_e1_bob_claim_wins_r4` / `phase_e2_alice_reclaim_wins_r4` — first-confirmed-spend-wins.
- `bob_discovers_three_payments_after_seed_restore` (recovery) — Bob recovers 3 from seed.
- `bob_claims_unspent_receipt_after_restore` — Bob claims after restore.
- `alice_payment_persists_across_restart` / `alice_reclaim_after_restart` — Alice persistence and reclaim.
- `alice_seed_only_recovery_with_empty_db` — Alice seed-only recovery with empty DB.

### Area 2: Consensus correctness
- `experiment_a_valid_bob_claim_is_accepted_by_consensus` — valid BobClaim accepted.
- `experiment_b1_alice_plain_spend_of_receipt_output` — Alice Plain spend accepted.
- `experiment_b2_alice_cannot_forge_bob_claim` — Alice cannot forge BobClaim.
- `experiment_b3_bob_plain_spend_of_receipt_output` — Bob Plain spend rejected.
- `experiment_b4_bob_plain_spend_without_receipt_metadata` — Bob without metadata rejected.
- `step3c_alice_reclaim_via_plain_kernel_consensus_accepted` — Alice reclaim accepted.
- `step3c_unauthorized_plain_spend_of_receipt_rejected` — unauthorized spend rejected.
- `step3c_mixed_branch_rejected_in_chain` — mixed branch rejected.
- `step3c_cut_through_bob_claim_preserves_receipt_input` — cut-through preserves receipt input.
- `step3c_cut_through_alice_spend_preserves_receipt_input` — cut-through preserves Alice input.
- `double_spend_rejection` (recovery) — double-spend rejected after claim.
- `payment_state_updates_after_reorg` (recovery) — reorg state reconciliation.

### Area 3: Cryptographic independence
- `alice_authorization_key_is_not_the_shared_blind_or_bob_receipt_key` (core).
- `three_payments_generate_distinct_alice_authorization_keys` (core).
- `authorization_public_key_must_match_the_receipt_hash` (core).
- `wrong_public_key_fails`, `wrong_commitment_fails`, `wrong_excess_fails`, `wrong_fee_fails`, `wrong_kernel_hash_fails`, `wrong_chain_identifier_fails` (core).
- `keys_are_domain_separated_and_unique` (wallet).
- `distinct_paths_are_used_for_each_key_type` (wallet).
- `descriptor_is_stable_for_same_seed` / `descriptor_differs_for_different_seed` (wallet).
- `no_key_reuse_across_payments_or_restarts` (recovery).

### Area 4: Documentation and reproducibility
- `PROTOCOL.md` — functional specification.
- `docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md` — design and threat model.
- `CURRENT_STEP.md` — step-by-step implementation record (3A–3G).
- `POC_RESULTS.md` — this document.
- `snip-wallet/tests/derivation.rs` — 10 unit tests.
- `chain/tests/snip_wallet.rs` — 12 Step 3D tests.
- `chain/tests/snip_wallet_recovery.rs` — 15 Step 3E/3G tests.

---

## 5. Test results (final)

| Command | Result |
|---|---|
| `cargo test -p epic_snip_wallet` | 10 passed |
| `cargo test -p epic_chain --test snip_wallet_recovery -- --test-threads=1` | 15 passed |
| `cargo test -p epic_chain --test snip_wallet -- --test-threads=1` | 12 passed |
| `cargo test -p epic_core --lib` | 84 passed |
| `cargo test -p epic_chain --test spend_attribution -- --test-threads=1` | 16 passed |
| `cargo test -p epic_chain --test receipt_claim_roundtrip -- --test-threads=1` | 1 passed |

---

## 6. Reproducible demonstration run

A single command runs the complete Step 3D demonstration (3 payments, claims,
reclaim, conflicts, restart):

```bash
cd upstream/epic && cargo test -p epic_chain --test snip_wallet -- --test-threads=1
```

The Step 3E/3G recovery and hardening tests:

```bash
cd upstream/epic && cargo test -p epic_chain --test snip_wallet_recovery -- --test-threads=1
```

The cryptographic core tests:

```bash
cd upstream/epic && cargo test -p epic_core --lib
```

---

## 7. Limitations and scope boundaries

### Design limitations
1. Alice reclaim requires the public recipient descriptor (receipt output does
   not commit to it). Strict seed+chain-only reclaim needs a receipt-format
   change.
2. Alice can reclaim any unclaimed payment at any time (first-confirmed-spend-
   wins). This is by design (PROTOCOL.md §3.9).
3. The shared blind `q_i` is known to both parties; it is a balance-construction
   capability, not an authorization.
4. Collusion between Alice and Bob is out of scope.

### Implementation limitations (Usernet prototype)
5. Activation is gated by process-global test flags, not height-based consensus.
6. JSON prototype store; production should use the Epic wallet database.
7. Single-writer (file lock); no concurrent writers.
8. 100-index recovery gap limit.
9. No height-based activation test.
10. Descriptor authenticity/distribution out of scope.

### What was NOT investigated
- Mainnet/Floonet compatibility.
- Production wallet integration (beyond adapter).
- UI, infrastructure, deployment.
- Performance optimization.
- Full seed recovery without additional metadata (descriptor required).
- Formal security proof (compact reasoning only).
- Network-level attacks (eclipse, Sybil).
- Side-channel resistance.

---

## 8. Conclusion

Epic SNIP is a **technically validated Proof of Concept**. The core question —
whether semi-non-interactive payments can be built on Epic Cash's existing
cryptography through targeted extensions — is answered affirmatively. The
implementation demonstrates:

- Reusable descriptors and offline payment creation.
- Chain-based discovery without communication.
- Independent Bob claim and Alice reclaim paths.
- Consensus-enforced authorization preventing bypass.
- Receipt-aware cut-through preserving evidence.
- Deterministic key derivation with domain separation.
- Wallet recovery from seed (Bob) and database (Alice).
- First-confirmed-spend-wins conflict resolution.

The PoC does **not** prove that Epic Cash supports semi-non-interactive
transactions without changes. It demonstrates that this functionality can be
realized through targeted, well-defined extensions to Epic Cash's existing
cryptographic primitives and consensus rules.

**No commit or push was performed. The PoC is complete.**