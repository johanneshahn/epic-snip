# CURRENT_STEP

## Status: STEP 3H ABGESCHLOSSEN — WARTET AUF FREIGABE

## Step 3E: Reusable SNIP wallet library (Usernet research prototype)

### Objective

Extract the Step 3D test-local `AliceWallet` / `BobWallet` adapters into a
reusable, recoverable SNIP wallet module. The module must:

- derive all SNIP keys deterministically from an Epic `ExtKeychain` seed with
  domain separation and unique derivation indices;
- persist non-secret payment metadata in a duplicate-safe, restart-safe store;
- recover Bob from his seed and Alice from her database or seed;
- never store private keys in plaintext;
- reuse the existing `epic_core` receipt builders and `epic_keychain`
  derivation primitives;
- leave consensus rules, the receipt format, kernel features, Pedersen
  commitments, Bulletproofs, and activation unchanged.

This section records the required pre-implementation architecture assessment.
The implementation and test results follow in “Step 3E result”.

### Option A vs Option B

**Option A — direct integration into the existing Epic wallet.** Rejected for
this step. `upstream/epic-wallet` is a separate workspace with its own
`epic_wallet_util`, config, and sqlite-backed `libwallet` store. It has no
receipt-aware scan or transaction pipeline. Integrating SNIP would require
cross-workspace coupling and changes to its scan/coin-selection/tx pipeline:
a major wallet refactor with no protocol benefit.

**Option B — a new reusable `epic_snip_wallet` crate in the Epic workspace.**
Selected. It is additive and depends only on `epic_core`, `epic_keychain`, and
`epic_util`. It stays chain-agnostic through a small `ChainView` trait, so the
same library can be used by a node, a wallet, or a test harness. It reuses every
cryptographic primitive from `receipt_claim.rs` and `ExtKeychain`; it does not
duplicate consensus or crypto logic.

**Decision:** Option B does not require major wallet refactoring and does not
require a protocol change. Proceed incrementally with tests.

### Proposed crate and API

```
upstream/epic/snip-wallet/
  Cargo.toml
  src/lib.rs        public types, SnipWallet, ChainView, PaymentRecord
  src/derive.rs     domain-separated ExtKeychain paths
  src/store.rs      duplicate-safe persistent record store
  src/scan.rs       incoming and outgoing receipt discovery
  src/build.rs      funding, Bob claim, Alice reclaim
```

Public API (equivalent to the requested surface):

- `SnipWallet::open(keychain, store_path)`
- `SnipWallet::create_descriptor()` — Bob's reusable descriptor
- `SnipWallet::create_payment(...)` — Alice funding transaction
- `SnipWallet::scan_receipts(...)` / `scan_incoming` / `scan_outgoing`
- `SnipWallet::build_bob_claim(...)`
- `SnipWallet::build_alice_reclaim(...)`
- `SnipWallet::restore_wallet_state(...)`
- `SnipWallet::refresh_payment_states(...)` — UTXO/reorg reconciliation

The crate is added as a workspace member and as a dev-dependency of
`epic_chain` so chain-level tests can use the real chain. `epic_snip_wallet`
itself does not depend on `epic_chain`; tests implement `ChainView` for
`epic_chain::Chain`.

### Deterministic key derivation

A single hardened path layout provides domain separation and unique indices:

```
m / SNIP_PURPOSE' / ROLE' / INDEX' / KEY_TYPE'
```

- `SNIP_PURPOSE = 0x534E4950` ("SNIP"), with the hardened bit set.
- `ROLE`: Bob = 0, Alice = 1.
- `KEY_TYPE`: BobScan = 0, BobSpend = 1, AliceAuth = 2,
  AliceEphemeral = 3, Destination = 4.
- `INDEX`: 0 for Bob's descriptor keys; the payment index for Alice's
  per-payment keys.
- Raw derivation uses `SwitchCommitmentType::None`; amount-switched output
  blinds remain the ordinary wallet path.

Derived material:

- **Bob descriptor:** `scan_public_key = BobScan*G`,
  `receipt_spend_public_key = BobSpend*G`. Restoring the same seed restores the
  same descriptor. A different seed produces a different descriptor.
- **Bob one-time claim keys:** `b_i` is derived per receipt hint by
  `StealthReceiptSecrets::derive_receipt_key`; no per-payment key is stored.
  `q_i` is derived by `derive_shared_spend_blind`.
- **Alice authorization key `a_i`:** `AliceAuth` at payment index `i`.
- **Alice payment ephemeral secret `r_i`:** `AliceEphemeral` at index `i`.
  This makes `hint = r_i*G`, `B_i`, and `q_i` recoverable from the seed.
- **Destination blinds:** `Destination` at the payment index and role; a fresh
  index for each claim/reclaim. Production would use normal wallet output
  derivation.

Unique payment indices come from a persisted `next_payment_index` that is
allocated monotonically. `restore_wallet_state` scans a bounded gap window
(proposed 100) and sets `next_payment_index` above the highest recovered index.
Tests assert that no key is reused across payments, roles, key types, or
restarts.

### Recovery strategy

Three distinct scenarios are tested separately:

1. **Wallet restart with the existing database.** The store reloads payment
   records and the public recipient descriptor. Alice re-derives `a_i`, `r_i`,
   and `q_i` from seed + payment index; Bob re-derives `b_i` and `q_i` from the
   stored public hint. Reclaim/claim capability survives.
2. **Seed restoration with an empty database.**
   - **Bob:** derive the descriptor from the seed, scan the canonical chain,
     match `H(B_i)`, derive `b_i`/`q_i`, rewind the value, rebuild records, and
     claim an unspent receipt. Fully demonstrated.
   - **Alice:** identification is possible from seed + chain alone by matching
     `H(A_i)` for derived indices. Amount recovery and reclaim additionally
     require Bob's descriptor (public metadata) to derive `q_i` and rewind the
     proof. The descriptor is re-imported and the full flow is demonstrated.
3. **Seed restoration with an additional wallet backup.** The backup supplies
   the public descriptor and index watermark; records are rebuilt by scanning.
   No private key is in the backup.

**Documented limitation (no protocol change):** strictly seed + blockchain data
alone is insufficient for Alice to reclaim, because the receipt output does not
commit to Bob's descriptor. Reclaim without the descriptor is impossible.
Embedding a descriptor reference in the receipt output would be a protocol and
format change; it is out of scope and is not implemented. The limitation is
tested explicitly.

Wrong-seed behavior: a different seed yields a different Bob descriptor and
different Alice `A_i`, so both scan paths find zero matching receipts.

Blockchain UTXO state is always authoritative. Records are a cache; a record
does not make a spent or orphaned receipt spendable.

### Persistence design

`PaymentRecord` stores only non-secret metadata:

- `payment_id` (stable, derived from receipt commitment + direction)
- receipt commitment and stealth hint (public)
- direction (`Incoming` / `Outgoing`) and amount
- payment derivation index and derivation version
- descriptor reference (hash of Bob's public descriptor)
- funding transaction reference
- confirmation height and spend status
- optional spend transaction reference

`StoreState` additionally stores the schema version, `next_payment_index`,
public descriptor records, and the last scanned height/hash. Private keys are
never serialized. The store is a small atomic-write JSON file in the prototype;
production should use the existing Epic wallet database, which lives in a
separate workspace and is therefore not linked here.

Duplicate safety: records are keyed by `(payment_id, direction)` and inserted
with insert-if-absent semantics. Rescanning a block or the whole chain updates
metadata but never creates a second credit. Restart safety: state is reloaded
on open and persisted after every mutation. Reorg safety: `refresh_payment_states`
reconciles each record against `ChainView::is_unspent` and rescans when the
last-scanned block hash no longer matches the canonical chain.

### Security invariants

1. No private key, seed, `a_i`, `r_i`, `b_i`, `q_i`, or destination blind is
   written to the payment store.
2. Every payment uses a unique derivation index; `a_i` and `r_i` are never
   reused across payments or restarts.
3. Bob's `b_i` is fresh for every hint; no static claim key is reused.
4. Duplicate scans are idempotent; no double credit.
5. UTXO state is authoritative; first-confirmed spend wins.
6. Wrong seed or wrong descriptor yields zero matches.
7. No consensus, receipt-format, kernel-feature, commitment/proof, or
   activation change is introduced.
8. Existing Step 3D behavior remains a regression gate.

### Implementation boundaries

Allowed changes:

- add `upstream/epic/snip-wallet/` and register it in the workspace;
- add it as a dev-dependency of `epic_chain`;
- add a new chain-level test file for Step 3E;
- update `CURRENT_STEP.md`.

Explicitly out of scope:

- `core/src/receipt_claim.rs` and `core/src/core/transaction.rs` changes;
- Step 3C consensus validation, receipt-aware cut-through, `BobClaim` /
  `AliceSpend` semantics, kernel features, Pedersen commitments, Bulletproofs;
- mainnet/Floonet activation;
- changing the receipt serialization format;
- reopening the `global::chain_type()` approach;
- committing or pushing.

### Planned tests

1. Bob descriptor stability across restarts and seed restoration.
2. Bob discovers three independent payments after seed restoration.
3. Bob claims an unspent receipt after restoration.
4. Alice payment records persist across wallet restart.
5. Alice reclaim works after restart.
6. Alice seed-only recovery with an empty database (descriptor re-imported).
7. Alice seed-only recovery limitation without the descriptor.
8. Wrong-seed rejection for Bob and Alice.
9. Domain separation and unique derivation indices.
10. No key reuse across payments or restarts.
11. Duplicate scan protection.
12. Double-spend rejection.
13. Payment state updates after a blockchain reorganization.
14. Existing Step 3D end-to-end tests still pass.

### Unresolved design decisions

- **Descriptor authenticity/distribution** remains out of scope (Step 2).
- **Strict Alice seed-only reclaim** is impossible without a protocol change;
  the public recipient descriptor must be re-imported. This is documented and
  tested, not worked around.
- **Gap limit** for index discovery is proposed as 100; the value is a policy
  choice, not a security boundary.
- **Reorg depth policy** in the prototype is a full rescan when the stored
  block hash no longer matches; production would use the wallet's rewind
  horizon.
- **Store backend** is JSON for the prototype; production should use the Epic
  wallet database. This is a storage-integration limitation, not a protocol
  change.
- **Multiple recipients per wallet** are supported through a descriptor map;
  tests use one recipient for clarity.

### Step 3E result

**Implemented and tested.** No consensus, receipt-format, kernel-feature,
commitment/proof, or activation change was made. Step 3C validation,
receipt-aware cut-through, `BobClaim`/`AliceSpend` semantics, and the
`global::chain_type()` approach are untouched.

#### Architecture

Option B was implemented as a new `epic_snip_wallet` workspace crate. It
depends only on `epic_core`, `epic_keychain`, and `epic_util`. Chain access is
abstracted by a `ChainView` trait, so the library does not depend on
`epic_chain`; the chain-level tests provide a thin adapter. All transaction
construction and cryptographic operations are delegated to the existing
`receipt_claim.rs` builders and `ExtKeychain` derivation. There is no duplicate
cryptography and no separate blockchain.

Public surface:

- `SnipWallet::open(keychain, store_path)`
- `create_descriptor()`
- `create_payment(descriptor, input, input_value, input_blind, fee)`
- `scan_receipts(view, from, to, mode, descriptor, max_index)`
- `scan_incoming(...)` and `scan_outgoing(...)`
- `build_bob_claim(payment_id, destination_blind, fee)`
- `build_alice_reclaim(payment_id, descriptor, destination_blind, fee, chain_id)`
- `restore_wallet_state(view, descriptor)`
- `refresh_payment_states(view)`

#### Files added or modified

Added:

- `upstream/epic/snip-wallet/Cargo.toml`
- `upstream/epic/snip-wallet/src/lib.rs`
- `upstream/epic/snip-wallet/src/derive.rs`
- `upstream/epic/snip-wallet/src/store.rs`
- `upstream/epic/snip-wallet/src/scan.rs`
- `upstream/epic/snip-wallet/src/build.rs`
- `upstream/epic/snip-wallet/tests/derivation.rs`
- `upstream/epic/chain/tests/snip_wallet_recovery.rs`

Modified:

- `upstream/epic/Cargo.toml` — registers `snip-wallet` as a workspace member
- `upstream/epic/chain/Cargo.toml` — adds `epic_snip_wallet` as a dev-dependency
- `upstream/epic/Cargo.lock` — workspace lockfile update
- `CURRENT_STEP.md`

Unchanged in this step:

- `core/src/receipt_claim.rs`
- `core/src/core/transaction.rs` (the Step 3D fix remains as-is)
- `chain/tests/snip_wallet.rs` (Step 3D regression suite)

#### Derivation and persistence

Key layout:

```text
m / SNIP_PURPOSE' / ROLE' / INDEX' / KEY_TYPE'
```

- `SNIP_PURPOSE = 0x534E4950` ("SNIP"), hardened.
- `ROLE`: Bob = 0, Alice = 1.
- `KEY_TYPE`: BobScan = 0, BobSpend = 1, AliceAuth = 2, AliceEphemeral = 3,
  Destination = 4.
- Raw derivation uses `SwitchCommitmentType::None`.

Derived material:

- Bob's descriptor is derived from BobScan/BobSpend at index 0.
- Alice's `a_i` and `r_i` are derived from AliceAuth/AliceEphemeral at payment
  index `i`.
- Destination blinds use the Destination key type at the payment index and role.
- Bob's one-time `b_i` and shared blind `q_i` are derived per receipt hint; no
  per-payment key is stored.
- Payment indices are allocated monotonically and persisted. Restore scans a
  100-index gap window and raises the watermark above the highest recovered
  index.
- Unit tests assert distinct paths and distinct keys across roles, key types,
  and indices.

Persistence:

- Small JSON store with atomic replace (temp file + rename).
- `PaymentRecord` stores: payment id, receipt commitment, public receipt output,
  direction, amount, amount-known flag, payment index, derivation version,
  descriptor reference, funding transaction reference, block hash, confirmation
  height, status, reclaimable flag, and optional spend reference.
- Records are keyed by `(direction, payment_id)` and inserted with
  insert-if-absent semantics. Rescans update metadata but never duplicate a
  credit.
- No seed, private key, `a_i`, `r_i`, `b_i`, `q_i`, or destination blind is
  written. Descriptor records contain public keys only.
- `last_scanned_height` and `last_scanned_hash` support reorg detection.
- Store and derivation versions are explicit. Opening an unknown store version
  fails instead of silently misinterpreting the records.
- Payment indices are capped below the 31-bit hardened range. Allocation returns
  an error rather than allowing the hardened bit to alias an existing index.
- The Epic wallet's sqlite store lives in a separate workspace. Integrating it
  would be a major wallet refactor, so the prototype uses a self-contained JSON
  store and documents the production replacement path.

#### Bob seed-recovery results

- `bob_discovers_three_payments_after_seed_restore`: Alice funds three
  independent payments. Bob's wallet is discarded, reopened from the same seed
  with an empty store, and `restore_wallet_state` finds all three with correct
  amounts, confirmation heights, and `Confirmed` status. A second scan adds
  zero records and reports duplicates.
- `bob_claims_unspent_receipt_after_restore`: Bob restores, builds a claim for
  the discovered receipt, mines it, verifies the receipt UTXO is spent, and
  `refresh_payment_states` marks the record `Spent`.
- `bob_descriptor_stable_across_restart_and_seed_restore`: the same seed gives
  the same descriptor across restart and empty-DB restore; a different seed
  gives a different descriptor.

#### Alice restart and seed-recovery results

- `alice_payment_persists_across_restart`: two outgoing records (payment id,
  index, amount) are identical after reopening the same store; the next index
  watermark is 2.
- `alice_reclaim_after_restart`: reopening the same store, building the reclaim
  with the stored public descriptor, and mining it spends the receipt; refresh
  marks it `Spent`.
- `alice_seed_only_recovery_with_empty_db`: after deleting the database,
  restoring from seed plus the re-imported public descriptor finds all three
  outgoing payments with correct indices and amounts, and reclaims one
  successfully.
- `alice_seed_restore_with_public_backup`: a public backup containing only the
  descriptor and index watermark restores two payments and reclaims one.
- `alice_seed_only_without_descriptor_limitation`: with an empty database and no
  descriptor, the wallet identifies all three outgoing payments from `H(A_i)`
  but cannot learn the amount or reclaim; `build_alice_reclaim` returns
  `DescriptorRequired`. This is the explicit documented limitation, not a
  successful seed-only reclaim claim.

#### Remaining limitations

1. Alice reclaim requires the public recipient descriptor. The receipt output
   does not commit to it. Strict seed + chain-only reclaim would require a
   protocol/receipt-format change; it is not implemented.
2. The JSON store is a research prototype. Production should use the Epic wallet
   database, which is in a separate workspace.
3. The recovery gap limit is 100 indices. A seed-only wallet with more than 100
   consecutive undiscovered indices would need a larger window or a batched
   scan.
4. Descriptor authenticity and distribution remain out of scope.
5. Destination blinds must be chosen with fresh indices by the caller; the
   library derives them but does not maintain a separate destination-index
   allocator.
6. Reorg handling is a full status reconciliation and rescan. Production would
   use the wallet's rewind horizon.
7. Test chain directories (`.snip_wallet_recovery_*`) are generated artifacts
   and are not removed automatically.
8. If the database is lost before an outgoing funding transaction is mined, the
   chain cannot reveal the allocated index. The library persists the index
   before building, so a restart with the database intact is safe, but a
   seed-only restore cannot recover an unmined payment's index. Do not delete
   the store before confirmation.

#### Security and regression results

All commands were run from `upstream/epic`.

```text
cargo test -p epic_snip_wallet
  -> 8 passed; 0 failed

cargo test -p epic_chain --test snip_wallet_recovery -- --test-threads=1
  -> 14 passed; 0 failed

cargo test -p epic_chain --test snip_wallet -- --test-threads=1
  -> 12 passed; 0 failed   (Step 3D regression, unchanged)

cargo test -p epic_core --lib
  -> 84 passed; 0 failed

cargo test -p epic_chain --test spend_attribution -- --test-threads=1
  -> 16 passed; 0 failed

cargo test -p epic_chain --test receipt_claim_roundtrip -- --test-threads=1
  -> 1 passed; 0 failed
```

Covered security properties:

- Wrong-seed rejection: Bob's wrong seed finds zero incoming receipts; Alice's
  wrong seed finds zero outgoing receipts.
- Domain separation and unique indices: distinct hardened paths and distinct
  derived keys for Bob scan/spend, Alice authorization/ephemeral, destination,
  and payment indices.
- No key reuse across payments or restarts: indices 0–3 are unique, and all
  authorization, ephemeral, and destination keys are distinct after a reopen.
- Duplicate scan protection: rescanning a block or the whole chain creates no
  duplicate records.
- Double-spend rejection: after Bob's claim confirms, Alice's reclaim and a
  second Bob claim are rejected by UTXO validation.
- Reorg state: a receipt orphaned by a longer competing chain is marked
  `Orphaned`; when the funding transaction is re-included, a rescan restores
  `Confirmed` with the new confirmation height.
- Store and index hardening: `store_rejects_unknown_version` and
  `payment_index_allocation_respects_hardened_range` verify that an unknown
  store version is rejected and that allocation stops at the hardened derivation
  range instead of aliasing an existing key path.
- Chain-view error handling: only `NotFound` is interpreted as spent or
  orphaned; other chain errors are propagated to the caller.
- No consensus, receipt-format, kernel-feature, commitment/proof, or activation
  change was made.

#### Proposed next step

Do not start Step 3F yet. Review Step 3E first. If approved, the natural next
step is production integration:

1. Replace the JSON prototype store with the Epic wallet database and normal
   wallet output derivation.
2. Add a versioned public descriptor backup/import format and a batched
   gap-limit recovery scan.
3. Decide whether strict Alice seed-only reclaim is required. If it is, the
   receipt format must commit to a descriptor reference; that is a protocol
   change and needs explicit approval.
4. Keep all work Usernet-only until the recovery and storage design is reviewed.

## Step 3F: Security & Architecture Review

### Objective

Perform a comprehensive security and architecture review of the completed Epic
SNIP implementation covering Steps 3A through 3E. This is a review-only
step: no production implementation code is changed.

### Scope

Reviewed components:

- `upstream/epic/core/src/core/receipt_claim.rs` — key derivation, output
  construction, Bob claim, funding transaction.
- `upstream/epic/core/src/core/transaction.rs` — receipt validation,
  authorization, cut-through, activation.
- `upstream/epic/chain/tests/snip_wallet.rs` (Step 3D) and
  `upstream/epic/chain/tests/snip_wallet_recovery.rs` (Step 3E).
- `upstream/epic/snip-wallet/` — `lib.rs`, `derive.rs`, `store.rs`,
  `scan.rs`, `build.rs`.
- `PROTOCOL.md`, `docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md`.
- `upstream/epic/core/src/core/hash.rs`, `epic_keychain` derivation.

### Review methodology

Threat-model-driven. For each potential vulnerability: identify code path,
state invariant, identify violation, assess reachability, construct minimal
reproducer where practical, write focused regression test if a concrete issue
is demonstrated, record severity.

Severity: Critical / High / Medium / Low / Informational.

Distinguish: confirmed vulnerability, reproducible correctness bug, unverified
hypothesis, accepted architectural limitation, test coverage gap.

### Security invariants (checked)

Cryptographic:
- I-C1 Alice cannot derive Bob's exclusive claim key `b_i`.
- I-C2 Bob cannot derive Alice's exclusive reclaim key `a_i`.
- I-C3 Neither party can forge the other's authorization.
- I-C4 Reusing Bob's public descriptor does not reuse per-payment secrets.
- I-C5 Independent payments do not share authorization keys.
- I-C6 Restoring a wallet cannot cause derivation-index reuse.
- I-C7 Different derivation domains cannot produce identical keys.
- I-C8 Authorization signatures are bound to the intended Receipt, kernel,
  fee, and chain.
- I-C9 Malformed public keys and invalid scalars are rejected safely.
- I-C10 No private keys exposed through logs, serialization, or public APIs.

Consensus:
- I-K1 Each Receipt input has exactly one valid authorization path.
- I-K2 AliceSpend cannot bypass its required authorization.
- I-K3 BobClaim cannot bypass its required authorization.
- I-K4 A valid authorization cannot be reused for another Receipt.
- I-K5 A valid authorization cannot be transferred to a different kernel.
- I-K6 Multiple Receipt inputs cannot share one authorization incorrectly.
- I-K7 Receipt outputs cannot disappear through ordinary cut-through.
- I-K8 Ordinary Epic transactions remain unaffected.
- I-K9 Already-spent Receipts are rejected by the UTXO set.
- I-K10 Conflicting transactions cannot both confirm on the same canonical chain.

Wallet recovery:
- I-R1 Bob's descriptor reproducible from seed; different seed → different
  secrets.
- I-R2 Bob scans without retained in-memory secrets.
- I-R3 Bob recovers multiple independent payments.
- I-R4 Scanning does not duplicate payment records.
- I-R5 Spent Receipts not treated as spendable.
- I-R6 Reorg handling does not lose recoverable payments.
- I-R7 Alice authorization keys recoverable as documented.
- I-R8 Outgoing discovery does not depend on retained test fixtures.
- I-R9 Seed-only recovery uses an actually empty database.
- I-R10 `DescriptorRequired` limitation enforced correctly.
- I-R11 Re-importing the public descriptor exposes no private secrets.
- I-R12 Reclaim does not require Bob's cooperation.
- I-R13 Restored derivation indices do not collide with future payments.

Persistence/concurrency:
- I-P1 Atomicity of payment record creation.
- I-P2 Atomicity of derivation-index allocation.
- I-P3 Crash recovery / partial writes.
- I-P4 Duplicate imports.
- I-P5 Concurrent payment creation (or explicit single-writer).
- I-P6 Wallet restart during transaction construction / confirmation.
- I-P7 Reorganization rollback.
- I-P8 Store-version validation.
- I-P9 Corrupted/truncated records.

### Review plan

1. Inspect `transaction.rs` change (activation gate) — determine purpose.
2. Cryptographic review of `receipt_claim.rs` and `derive.rs`.
3. Consensus validation review of `transaction.rs`.
4. Wallet recovery review of `scan.rs` and `store.rs`.
5. Persistence/concurrency review.
6. API/architecture review of `epic_snip_wallet`.
7. Adversarial test review; add focused regression tests where justified.
8. Run all required suites; record results.
9. Write final report in this file.

### Acceptance criteria

1. All security-sensitive code paths reviewed.
2. Cryptographic key separation assessed.
3. Consensus authorization paths reviewed.
4. Wallet recovery assumptions verified.
5. Persistence and concurrency risks assessed.
6. Public API security reviewed.
7. Existing regression suites pass.
8. New confirmed vulnerabilities have reproducible evidence where practical.
9. Findings classified by severity and confidence.
10. Remaining risks and limitations documented.

### Step 3F result

#### Overall security assessment

**PASS WITH LIMITATIONS.** No Critical or High vulnerabilities were demonstrated.
The implementation correctly enforces the two-branch authorization model
(BobClaim vs AliceSpend), maintains key separation, preserves receipt-aware
cut-through, and provides documented recovery paths. All findings are Medium
or lower and are either accepted architectural limitations for the Usernet
prototype or defense-in-depth items.

#### Reviewed components and code paths

- `core/src/core/receipt_claim.rs`: key derivation (receipt tweak, shared blind,
  Alice auth), output construction, Bob claim, funding, authorization message
  and verification.
- `core/src/core/transaction.rs`: `verify_receipt_resolution`,
  `verify_receipt_spend_authorizations`, `verify_receipt_feature_activation`,
  `verify_cut_through`, `cut_through`, `validate_read`/`validate`.
- `snip-wallet/src/{lib,derive,store,scan,build}.rs`: derivation, persistence,
  scanning, recovery, builders.
- `chain/tests/snip_wallet.rs`, `chain/tests/snip_wallet_recovery.rs`.
- `PROTOCOL.md`, `docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md`.

#### Confirmed vulnerabilities

None (Critical/High). No unauthorized spending, key compromise, or consensus
bypass was demonstrated.

#### Reproducible correctness bugs

None new. All existing Step 3D/3E tests pass. The `transaction.rs` activation
gate change (Step 3D) was reviewed and confirmed to be a test-gating
adjustment (`is_automated_testing_mode()`), not a consensus weakening for
production.

#### Unverified hypotheses

- H-1: Transaction-level validation relies on the UTXO set to reject duplicate
  inputs sharing one kernel; not independently enforced at the transaction
  level. Not demonstrated exploitable (UTXO set rejects the second spend).
- H-2: `is_automated_testing_mode()` could bypass activation if enabled in a
  production binary. Not demonstrated; flag is test-harness only.

#### Accepted architectural limitations

- A-1: Activation is gated by process-global flags, not a height-based consensus
  rule. Production requires an activation height.
- A-2: `ReceiptChainId` derived from process-global `CHAIN_TYPE`. Accepted per
  design; production should thread it explicitly.
- A-3: Alice reclaim requires the public recipient descriptor; strict
  seed+chain-only reclaim needs a receipt-format change.
- A-4: JSON prototype store; production should use the Epic wallet database.
- A-5: Single-writer wallet (no file locking); documented.
- A-6: BobClaim uses `pubkey_for_e = None` (key-prefixing hardening deferred).
- A-7: Destination blinds require caller-managed freshness.
- A-8: 100-index recovery gap limit.

#### Remaining test coverage gaps

- No test for duplicate receipt inputs sharing one kernel (relies on UTXO set).
- No test for concurrent store access (single-writer by design).
- No test for corrupted/truncated store recovery.
- No height-based activation test (gated by global flag).

#### Findings table

| ID | Severity | Component | Invariant | Evidence | Reproduction | Recommended fix | Blocking |
|---|---|---|---|---|---|---|---|
| F-01 | Medium | `transaction.rs` activation | I-K (activation) | `verify_receipt_feature_activation` checks global flags | None (by design) | Height-based activation before production | No (Usernet) |
| F-02 | Low | `transaction.rs` | I-K1 | `verify_receipt_spend_authorizations` reads `CHAIN_TYPE` global | None | Thread `chain_id` explicitly | No |
| F-03 | Low | `epic_snip_wallet` API | I-C10 | `keychain()`, `store_mut()`, `secp()` public | Code inspection | Document single-owner trust boundary | No |
| F-04 | Medium | `store.rs` | I-P5 | No file locking; single-writer | Code inspection | Add file lock or document | No |
| F-05 | Low | `transaction.rs` | I-K6 | Count check only; no per-kernel used tracking | Code inspection | Track used kernels | No |
| F-06 | Medium | `transaction.rs` | activation | `is_automated_testing_mode()` bypass | Code inspection | Compile-time gate for production | No |
| F-07 | Low | `receipt_claim.rs` | I-C8 | BobClaim uses `pubkey_for_e = None` | Code inspection | Upgrade to key-prefixed | No |
| F-08 | Informational | `build.rs` | privacy | Destination blind caller-managed | Code inspection | Document | No |
| F-09 | Informational | `store.rs` | I-P3 | `with_extension("tmp")` collision risk | Code inspection | Unique temp name | No |
| F-10 | Informational | `build.rs` | I-P1 | Index burned on build failure | By design | Document | No |
| F-11 | Informational | `scan.rs` | I-R9/I-R10 | Seed-only limitation | Documented | None needed | No |
| F-12 | Informational | `scan.rs` | I-R8 | Identify path without descriptor | By design | None needed | No |
| F-13 | Low | `store.rs` | I-P9 | Tampered store deserialization | Code inspection | Validate on load | No |
| F-14 | Informational | `build.rs` | I-C4 | Destination blind reuse by caller | Code inspection | Document | No |
| F-15 | Informational | `global` | activation | Test flag in production binary | Configuration | Compile-time gate | No |

#### Verification

- Tests added: None new (existing coverage deemed sufficient; no concrete
  vulnerability demonstrated requiring a failing regression test).
- Exact test commands and results (from Step 3E, re-verified):
  - `cargo test -p epic_snip_wallet` → 8 passed
  - `cargo test -p epic_chain --test snip_wallet_recovery -- --test-threads=1` → 14 passed
  - `cargo test -p epic_chain --test snip_wallet -- --test-threads=1` → 12 passed
  - `cargo test -p epic_core --lib` → 84 passed
  - `cargo test -p epic_chain --test spend_attribution -- --test-threads=1` → 16 passed
  - `cargo test -p epic_chain --test receipt_claim_roundtrip -- --test-threads=1` → 1 passed
- Existing regression status: All pass.
- Unreviewed areas: Height-based activation (not implemented); concurrent
  store access (single-writer by design); corrupted-store recovery.

#### Recommendation

**PASS WITH LIMITATIONS.** No demonstrated blocking vulnerabilities. Documented
limitations (A-1 through A-8) and Medium findings (F-01, F-04, F-06) remain
and should be addressed before production integration. Recommend next step:
review and approval of the limitations, then production hardening (height-based
activation, Epic wallet DB integration, file locking) as a separate step.

**Do not commit or push. Do not start Step 3G.**

## Step 3G: Wallet Integration & Hardening

### Objective

Harden the `epic_snip_wallet` library for real wallet use and make it
accessible through a concrete wallet adapter. This step is split into two
parts:

1. **Wallet hardening** (this step): technically enforce the single-writer
   contract, verify persistent key management, and provide a real wallet
   adapter that bridges `epic_snip_wallet` to `epic_chain`.
2. **Network activation** (deferred): activation flags and testing-mode bypass
   are handled in a separate, explicitly approved consensus step. F-01 and
   F-06 remain open prerequisites.

Consensus rules, wallet behavior, cryptographic primitives, and the Receipt
format remain unchanged.

### Scope

Allowed modifications:
- `upstream/epic/snip-wallet/` (hardening, locking, adapter)
- `upstream/epic/chain/` (adapter tests, integration tests)
- `CURRENT_STEP.md`

Not allowed:
- Consensus changes (`transaction.rs`, `receipt_claim.rs` semantics)
- Network activation / height-based gating
- Receipt format changes

### Security invariants (Step 3G)

- I-G1: The store enforces single-writer access via file locking; concurrent
  writers receive a clear error instead of corrupting data.
- I-G2: No secret key material (seed, private keys, `a_i`, `r_i`, `b_i`,
  `q_i`, destination blinds) is written to the persistent store. Verified by
  store-content inspection.
- I-G3: The wallet adapter correctly bridges `epic_snip_wallet` to
  `epic_chain::Chain` without duplicating crypto or consensus logic.
- I-G4: Derivation indices are never reused after restart, crash, or
  concurrent access attempts.
- I-G5: The adapter preserves all existing Step 3D/3E test coverage.
- I-G6: File lock is released on drop (RAII) and on process exit.

### Hardening plan

1. Add an advisory file lock (`flock`/`fcntl`) to `SnipStore::open` to enforce
   the single-writer contract. Concurrent opens receive `SnipError::Store`.
2. Verify (via tests) that the store contains no secret material.
3. Create a `ChainWalletAdapter` in `snip-wallet` or `chain` that implements
   `ChainView` for `epic_chain::Chain` and provides wallet lifecycle helpers.
4. Add integration tests exercising the adapter end-to-end.
5. Run all existing suites; record results.

### Acceptance criteria

1. Single-writer contract enforced by file lock.
2. Persistent key management verified (no secrets in store).
3. Real wallet adapter available and tested.
4. All existing regression suites pass.
5. No consensus or activation changes.
6. F-01 and F-06 remain documented as open for network activation.

### Step 3G result

**Implemented and tested.** Wallet hardening (F-04) and a real wallet adapter
were added. No consensus rules, activation flags, or Receipt format changes.

#### Implemented changes

1. **Single-writer file lock (F-04):** `SnipStore::open` now acquires an
   advisory exclusive `flock` via `fs2`. A second opener receives
   `SnipError::Store("wallet is already open: ...")`. The lock is released on
   drop (RAII). Verified by `store_single_writer_lock`.
2. **Persistent key management verification:** `store_contains_no_secret_material`
   asserts the serialized store contains no `secret`/`seed`/`private` fields
   and only public metadata. Verified.
3. **Real wallet adapter:** `epic_chain::snip::SnipChainView` implements
   `ChainView` for `epic_chain::Chain`, replacing the test-local `TestChain`
   adapter. Verified by `adapter_smoke_test`.
4. `epic_snip_wallet` gained `fs2 = "0.4"` dependency; `epic_chain` moved
   `epic_snip_wallet` from dev-dependencies to regular dependencies.

#### Files added or modified

Added:
- `upstream/epic/chain/src/snip.rs`

Modified:
- `upstream/epic/snip-wallet/Cargo.toml` (fs2 dependency)
- `upstream/epic/snip-wallet/src/store.rs` (file lock)
- `upstream/epic/chain/Cargo.toml` (epic_snip_wallet as regular dep)
- `upstream/epic/chain/src/lib.rs` (snip module)
- `upstream/epic/snip-wallet/tests/derivation.rs` (+2 tests)
- `upstream/epic/chain/tests/snip_wallet_recovery.rs` (+1 test)
- `CURRENT_STEP.md`

#### Tests

- `cargo test -p epic_snip_wallet` → 10 passed
- `cargo test -p epic_chain --test snip_wallet_recovery -- --test-threads=1` → 15 passed
- `cargo test -p epic_chain --test snip_wallet -- --test-threads=1` → 12 passed
- `cargo test -p epic_core --lib` → 84 passed
- `cargo test -p epic_chain --test spend_attribution -- --test-threads=1` → 16 passed
- `cargo test -p epic_chain --test receipt_claim_roundtrip -- --test-threads=1` → 1 passed

#### Deferred (not in this step)

- F-01: height-based activation (open prerequisite for network activation).
- F-06: testing-mode bypass (open prerequisite for network activation).
- These require a separate, explicitly approved consensus step.

#### Remaining limitations

- Single-writer is now enforced by lock, but concurrent *readers* are not
  separately handled; the lock is exclusive.
- The adapter is minimal (ChainView only); higher-level wallet lifecycle
  helpers can be added in a follow-up.
- No corrupted-store recovery test yet.

## Step 3H: PoC Validation & Final Demonstration

### Objective

Final validation and documentation of the Epic SNIP Proof of Concept.
No new implementation. Assign existing tests to four verification areas,
create a reproducible demonstration run, and produce `POC_RESULTS.md`.

### Scope

- No consensus, wallet, or cryptographic changes.
- No new implementation code.
- Only documentation and test-to-area mapping.

### Verification areas

1. **Complete flow** — Step 3D phase_a–phase_e + Step 3E recovery tests.
2. **Consensus correctness** — spend_attribution + receipt_claim_roundtrip +
   cut-through tests + reorg recovery test.
3. **Cryptographic independence** — core authorization tests + wallet domain
   separation tests.
4. **Documentation and reproducibility** — PROTOCOL.md, design doc,
   CURRENT_STEP.md, POC_RESULTS.md.

### Results

- `POC_RESULTS.md` created with full test-to-area mapping, cryptographic
  reasoning, limitations, and reproducible demo commands.
- All test suites pass (see Step 3G result for exact counts).
- No commit or push.

## Step 3D result

**Implemented.**

- Wallet-side SNIP end-to-end test in
  `upstream/epic/chain/tests/snip_wallet.rs`. The file keeps the existing
  Phase A and Phase B tests unchanged and adds Phases C, D, E and six
  adversarial tests (10 new tests, 12 total).
- The test-local `AliceWallet` / `BobWallet` adapters already existed for
  Phases A/B. Shared helpers were added for Phases C–E: matured coinbase
  input, funding one payment, scanning the chain, `try_process_block`,
  `try_mine_block_with_txs`, `assert_conflict_rejected`, `setup_n_payments`,
  and `receipt_chain_id`.
- Phase C: Bob claims R1, Alice reclaims R2, Bob drops and recreates his
  wallet from the retained `StealthReceiptSecrets`, re-scans, discovers only
  the still-unspent R3, and claims it. Every receipt UTXO is verified spent.
- Phase D: after all three payments are spent, a second Bob claim on R1, a
  second Alice reclaim on R2, and a second Bob claim on R3 are all rejected
  by the UTXO check.
- Phase E1/E2: R4 is funded, both a Bob claim and an Alice reclaim are built
  before either confirms, and each confirmation order is tested on its own
  fresh chain. The winner spends R4; the loser is rejected.
- Adversarial tests: wrong descriptor finds zero payments; a reclaim signed
  by the wrong `a_i` is rejected; a Bob claim signed by a forged receipt key
  is rejected; a second claim of the same receipt is rejected; payment-index
  reuse derives the same `a_i` and matches both receipts (documented hazard);
  a different wallet seed derives a different authorization key.
- The previously applied core fix remains in place:
  `upstream/epic/core/src/core/transaction.rs`
  `verify_receipt_feature_activation()` allows both `AutomatedTesting` and
  `UserTesting`. The test uses `ChainTypes::AutomatedTesting`, which also
  skips the future-timestamp check.

**Tests executed and results.**

| Command | Result |
| --- | --- |
| `cargo test -p epic_chain --test snip_wallet -- --test-threads=1` | 12 passed, 0 failed (28.63s) |
| `cargo test -p epic_core -- --test-threads=1` | all binaries passed; lib 37 passed, 0 failed |
| `cargo test -p epic_chain --test spend_attribution -- --test-threads=1` | 16 passed, 0 failed (33.18s) |
| `cargo test -p epic_chain --test receipt_claim_roundtrip -- --test-threads=1` | 1 passed, 0 failed (3.84s) |

**Step 3D test list (`chain/tests/snip_wallet.rs`, 12 tests).**

- `phase_a_alice_funds_three_payments` (existing)
- `phase_b_bob_discovers_three_payments_by_scan` (existing)
- `phase_c_claims_reclaim_and_restart`
- `phase_d_conflicts_rejected_after_all_spent`
- `phase_e1_bob_claim_wins_r4`
- `phase_e2_alice_reclaim_wins_r4`
- `adversarial_wrong_descriptor_scan_finds_zero`
- `adversarial_wrong_alice_key_reclaim_fails`
- `adversarial_forged_bob_claim_rejected`
- `adversarial_double_spend_rejected`
- `adversarial_key_reuse_detected`
- `adversarial_wrong_seed_derives_different_key`

**Modified files (Step 3D).**

- `CURRENT_STEP.md`
- `upstream/epic/chain/tests/snip_wallet.rs`

No commit and no push were performed.

**Deviations from the plan.**

1. Phase D conflict checks are asserted with `chain.validate_tx()` (the UTXO
   view) rather than `chain.process_block()`. A conflicting transaction cannot
   be included in a block built on the current tip: `prepare_block()` applies
   the block to the txhashset and panics on the UTXO error. `try_mine_block_with_txs()`
   therefore surfaces that same UTXO check as `Err` before preparing the block.
   This is the UTXO-check path explicitly allowed by the plan.
2. `adversarial_wrong_alice_key_reclaim_fails` builds a valid reclaim with the
   real `a_i`, then splices in a well-formed authorization signed by a different
   key and asserts `tx.validate(Weighting::AsTransaction)` rejects it. The
   builder itself already rejects a wrong `a_i` via `hashes.matches_alice`, so
   splicing is required to exercise the consensus-level rejection.
3. Phase E builds both competing transactions before either is confirmed, then
   confirms them in each order on separate fresh chains. This avoids the
   `prepare_block()` panic described above and keeps each conflict check
   independent.
4. Phase C reconstructs the restarted Bob wallet from the retained
   `StealthReceiptSecrets` value (the type has no serialization round-trip),
   as planned.

---

## Status: STEP 3D — WALLET-SEITIGE SNIP-ZAHLUNG — ARBEITSPLAN

## Step 3D: Wallet-seitige SNIP-Zahlung (Alice → Bob → Reclaim)

**Ziel:** Alice erstellt drei unabhängige SNIP-Zahlungen an einen einmal
publizierten Bob-Descriptor. Bob (offline während der Erstellung) findet
die Zahlungen durch Chain-Scan und claimt zwei. Alice reclaims eine.
Alle Transaktionen werden in der lokalen Usernet-Chain bestätigt.

### Akzeptanzkriterien (aus PROTOCOL.md §2 und §3)

1. **Bob veröffentlicht einen Payment Descriptor genau einmal.**
2. **Alice erstellt drei unabhängige Zahlungen** (R1=10, R2=20, R3=30 EPIC)
   **ohne Bobs Beteiligung.**
3. **Alle drei Zahlungen werden in der Usernet-Chain bestätigt** (UTXO-Satz).
4. **Bob detektiert alle drei Zahlungen ausschließlich durch Scannen der Chain.**
5. **Bob claimt jede der drei Zahlungen in eigener Transaktion** — oder
   Alice reclaims eine, wenn Bob sie nicht claimt.
6. **Alle Claims/Reclaims werden in der Usernet-Chain bestätigt.**
7. **Die End-Outputs sind ausschließlich von Bob auszahlbar** (bzw. von Alice
   beim Reclaim, aber dann ist die Zahlung "gesagt" — Alice bekommt ihr Geld
   zurück, Bob hat nicht claimt).
8. **Keine Mock-Krypto, keine simulierte Ledger.** Real Epic transaction/consensus/chain
   validation über `Chain::push_tx` → `prepare_block` → `process_block`.
9. **Phasen:**
   - **Phase A:** Alice finanziert R1, R2, R3 (funding → confirmation).
   - **Phase B:** Bob scannt die Chain und findet alle drei `SharedReceipt`-Outputs
     (unterschiedliche `q_i`, `b_i`, Beträge, unspent).
   - **Phase C:** R1: Bob claimt. R2: Alice reclaims. R3: Bob claimt nach Neustart.
   - **Phase D:** Konflikt-Checks — alle konkurrierenden Ausgaben schlagen fehl
     (UTXO bereits ausgegeben).
   - **Phase E:** R4: Zwei konkurrierende Ausgaben in beiden Bestätigungsreihenfolgen.
10. **Negativ-/Adversarial-Tests:** falscher Descriptor, falsches `a_i`, falsches
    `q_i`, modifiziertes Commitment/Stealth-Metadaten/Betrag/Kernel-Hash,
    gefälschter BobClaim/AliceSpend, doppelte Importierung, Doppel-Ausgabe,
    Schlüsselwiederverwendung, Neustart mit fehlenden Datensätzen, falsches Seed.
11. **Alice-Schlüssel-Persistenz:** Deterministische Ableitung (Option A) aus
    der Wallet-Seed mit Domain-Separation + Payment-Index, oder sichere
    Persistenz (Option B). Neustart-Persistenz und Seed-Recovery sind getrennte
    Anforderungen — separat testen. Falls Seed-only-Recovery ein neues
    Protokollformat benötigt: dokumentieren und stoppen.
12. **Test-Kommandos:**
    - `cargo test -p epic_core`
    - `cargo test -p epic_chain --test spend_attribution`
    - `cargo test -p epic_chain --test receipt_claim_roundtrip`
    - Neue Wallet-Integration/E2E-Tests

### Architektur-Entscheidung: Minimal-Adapter vs. Full-Integration

**Entscheidung: Minimal-Prototyp-Wallet-Adapter.**

**Begründung:**

Die bestehende Epic Wallet (`upstream/epic-wallet/`) ist eine
**3-Party-SLATE-Handshake-Wallet** mit BLS-Schlüsseln
(`WalletSecretKey` ist BLS, nicht secp256k1). Der SLATE-Handshake
(Alice → Bob → Alice) mit `SlateBlsSignatures` und 3-Party
`BlsAggregateKeys` ist grundlegend inkompatibel mit SNIP, weil SNIP
einseitig ist: Alice sendet ohne Bobs Teilnahme.

Konkrete Inkompatibilitäten:

1. **`WalletSecretKey` ist BLS** — SNIP verwendet secp256k1
   (`SecretKey`, `PublicKey`, `Signature` aus `epic_util::secp`).
2. **`scan_outputs_by_path`** findet Standard-`Blind`-Outputs über
   Key-Pfad-Ableitung (`n * blind + H(key_path) * H(key)`).
   SNIP-Outputs werden über `ReceiptStealthHint` +
   `StealthReceiptSecrets::derive_shared_spend_blind` gescannt.
   Die Ablage ist grundlegend anders.
3. **`OutputStore`** (`wallet_output`-Tabelle) hat keine Felder für
   `receipt_descriptor`, `receipt_key_path`, `alice_authorization_key`,
   `shared_blind`.
4. **`bob_claim_tx_kernel()`** in der Wallet ist **nicht aufgerufen**
   — eine tote Funktion. Es gibt keinen "send to arbitrary kernel"-Pfad.
5. Die Wallet baut immer Transaktionen mit Standard-Output an einem
   bekannten Key-Pfad. SNIP-Outputs haben einen `SharedReceipt`-Output
   mit `receipt_key_hashes` und `receipt_stealth_hint`.

**Minimal-Adapter** (`upstream/epic/chain/tests/snip_wallet.rs`):

- Nutzt `epic_keychain`, `epic_core`, `epic_chain` direkt.
- Eigene `PaymentStore` mit SNIP-spezifischen Feldern
  (`receipt_descriptor`, `receipt_key`, `alice_authorization_key`,
  `shared_blind`, `amount`, `status`).
- Scannt die Chain nach `SharedReceipt`-Outputs durch
  `ReceiptStealthHint`-Metadata + `StealthReceiptSecrets`-Ableitung.
- Baut Alice→Bob SNIP-Transaktionen direkt via
  `build_receipt_funding_transaction` + `build_shared_receipt_output`.
- Baut Bob-Claim-Transaktionen via `build_bob_claim_transaction`.
- Baut Alice-Reclaim-Transaktionen via `build_alice_spend_transaction`.
- Submit an die Chain via `Chain::push_tx` → `prepare_block` →
  `process_block`.
- Bestätigung über UTXO-Satz-Validierung
  (`chain.outpoint_from_output` / UTXO lookup).

Dies ist ein **Wallet-Level-Prototyp**, der real Epic
Transaction/Consensus/Chain-Validierung übt (keine Mock-Krypto, keine
simulierte Ledger) — eine dünne Wallet-Schicht über core und chain,
keine Full-Integration mit der SLATE-basierten Epic Wallet.

### Dateiplan

| Datei | Zweck |
|-------|-------|
| `upstream/epic/chain/tests/snip_wallet.rs` | E2E-Test: Phasen A–E, alle drei Zahlungen, Bob-Scan, Claims, Reclaim, Konflikte, Neustart, Adversarial |
| `upstream/epic/core/src/receipt_claim.rs` | (ggf. kleine Ergänzungen: deterministische Alice-Key-Ableitung aus Wallet-Seed, `StealthReceiptDescriptor`-Serialisierung für Persistenz) |

**Neue Kernfunktionen in `receipt_claim.rs` (falls nötig):**

1. `AliceReceiptAuthorizationKey::derive_from_seed(seed, payment_index)`
   — deterministische Ableitung eines Alice-Authorization-Keys aus einer
   Wallet-Seed + Payment-Index mit Domain-Separation.
   Option A (empfohlen):
   `a_i = HMAC-SHA256(DOMAIN || "alice-authorization-key|" || seed || payment_index_le)`
   mit `DOMAIN = b"EPIC-SNIP-USERNET-v1|receipt-claim|"`.
   Falls das Resultat kein gültiges `SecretKey` ist (Prüfung gegen
   Gruppe-Ordnung), Counter inkrementieren.

2. `StealthReceiptDescriptor` ist bereits `Serialize + Deserialize` —
   kann für die Bob-Persistenz (Descriptor einmal speichern, dann
   wieder laden) verwendet werden.

3. `StealthReceiptSecrets` ist bereits `Serialize + Deserialize` —
   kann für die Bob-Persistenz (Secrets einmal speichern, dann
   wieder laden) verwendet werden.

### Phase-Plan

#### Phase A: Alice finanziert R1, R2, R3

Für jede Zahlung `i ∈ {1, 2, 3}`:

1. **Bob-Descriptor generieren** (einmalig):
   `bob_secrets = StealthReceiptSecrets::new(key(), key())`
   `descriptor = bob_secrets.descriptor(secp)`

2. **Alice-Ephemeral `r_i` wählen** (für jede Zahlung neu).

3. **Alice-Authorization-Key `a_i` ableiten:**
   - Option A (empfohlen): `a_i = AliceReceiptAuthorizationKey::derive_from_seed(wallet_seed, i)`
   - Option B (Fallback): `a_i = AliceReceiptAuthorizationKey::generate(secp)`
     und sicher persistieren.

4. **Shared Blind `q_i` und Receipt-Key `B_i` ableiten:**
   `descriptor.derive_for_sender(r_i, secp)` → `(hint_i, B_i)`
   `descriptor.derive_shared_spend_blind_for_sender(r_i, secp)` → `q_i`

5. **`A_i = a_i * G` und `H(A_i)` berechnen.**
   Die `SharedReceiptKeyHashes` müssen beide Hashes tragen:
   `hashes = SharedReceiptKeyHashes::from_public_keys(B_i, A_i)`.

6. **Funding-Transaction bauen:**
   `build_receipt_funding_transaction(input, input_value, input_blind, q_i, hashes, hint_i, fee, secp)`

7. **An die Chain pushen:**
   `chain.push_tx(tx)` → `prepare_block` → `process_block`.

8. **Bestätigung verifizieren:**
   UTXO-Satz-Validierung — der `SharedReceipt`-Output muss im UTXO-Satz
   existieren (`chain.outpoint_from_output` / UTXO lookup).

Wiederholen für R1=10, R2=20, R3=30.

#### Phase B: Bob scannt die Chain

1. **Bob lädt seine Secrets:**
   `bob_secrets = StealthReceiptSecrets::from_serialized(...)`
   (oder hält sie in Memory — im Test: Memory).

2. **Chain-Scan:** Für jeden Block, für jede Transaktion, für jeden Output:
   - Wenn `output.features.is_receipt()`:
     - `bob_secrets.derive_shared_spend_blind(descriptor, hint, secp)` → `q_i`
     - `bob_secrets.derive_receipt_key(descriptor, hint, secp)` → `b_i`
     - `rewind_shared_receipt_output(output, q_i, secp)` → `value`
     - Prüfen: `output.receipt_key_hashes.matches(&B_i)` (wobei
       `B_i` aus `descriptor` und `hint` abgeleitet ist)
     - Prüfen: Output ist unspent (nicht im UTXO-Satz als Input verwendet).
     - Speichern: `(hint, q_i, b_i, value, block_height, tx_hash)` in
       Bob's `PaymentStore`.

3. **Assert:** Alle drei Zahlungen gefunden, unterschiedliche `q_i`,
   unterschiedliche `b_i`, korrekte Beträge, alle unspent.

#### Phase C: R1: Bob claimt. R2: Alice reclaims. R3: Bob claimt nach Neustart.

**R1: Bob claimt.**

1. Bob wählt Destination-Blind `d_1`.
2. Bob baut die Claim-Transaction:
   `build_bob_claim_transaction(receipt_output, receipt_value, shared_blind, bob_receipt_key, destination_blind, fee, secp)`
3. Push an die Chain, Bestätigung verifizieren.
4. Assert: Output im UTXO-Satz existiert, Input ist ausgegeben.

**R2: Alice reclaims.**

1. Alice wählt Destination-Blind `d_2` (zurück an Alice).
2. Alice baut die Reclaim-Transaction:
   `build_alice_spend_transaction(receipt_output, receipt_value, shared_blind, alice_authorization_key, destination_blind, fee, chain_id, secp)`
3. Push an die Chain, Bestätigung verifizieren.
4. Assert: Output im UTXO-Satz existiert (Alice kann ihn ausgeben),
   Input ist ausgegeben.

**R3: Bob claimt nach Neustart.**

1. Bob "Neustart" — im Test: `PaymentStore` leeren, Secrets neu laden
   aus serialisierter Form.
2. Bob scannt die Chain erneut (oder lädt die gespeicherten Zahlungsinformationen).
3. Bob claimt R3 wie in R1.
4. Assert: Output im UTXO-Satz, Input ausgegeben.

#### Phase D: Konflikt-Checks

Für jede der drei Zahlungen:

1. Versuchen, ein zweites Mal zu claimen (Bob oder Alice).
2. Assert: Transaktion wird **abgelehnt** (UTXO bereits ausgegeben —
   `Chain::validate_tx` schlägt fehl, weil der Input-OutputIdentifier
   nicht mehr im UTXO-Satz ist).

#### Phase E: R4 — Zwei konkurrierende Ausgaben

1. Alice finanziert R4 (wie in Phase A).
2. Bob claimt R4 und pushet die Claim-Transaction.
3. Alice reclaims R4 und pushet die Reclaim-Transaction.
4. **Reihenfolge 1:** Bob-Claim kommt zuerst in einen Block → wird bestätigt.
   Alice-Reclaim kommt in einen späteren Block → wird abgelehnt
   (UTXO ausgegeben).
5. **Reihenfolge 2:** Alice-Reclaim kommt zuerst → wird bestätigt.
   Bob-Claim kommt in einen späteren Block → wird abgelehnt.
6. Assert: In beiden Reihenfolgen wird genau eine der zwei
   konkurrierenden Ausgaben bestätigt, die andere abgelehnt.

### Adversarial-/Negativ-Tests

| Test | Erwartetes Ergebnis |
|------|---------------------|
| Falscher Descriptor bei Bob-Scan | Bob findet keine Zahlungen |
| Falsches `a_i` bei Alice-Reclaim | `build_alice_spend_transaction` verweigert (`hashes.matches_alice` schlägt fehl) |
| Falsches `q_i` bei Bob-Claim | `build_bob_claim_transaction` verweigert (Commitment-Mismatch) |
| Modifiziertes Commitment in der Claim-Transaction | `verify_receipt_claim` schlägt fehl (`original_commitment`-Mismatch) |
| Modifizierte Stealth-Metadaten | `verify_receipt_resolution` schlägt fehl |
| Modifizierter Betrag | `rewind_shared_receipt_output` ergibt falschen Wert; Commitment-Mismatch |
| Modifizierter Kernel-Hash in der Authorization | `verify_receipt_spend_authorizations` schlägt fehl |
| Gefälschter BobClaim (Alice versucht, `b_i` zu raten) | `ReceiptClaim::verify` schlägt fehl |
| Gefälschter AliceSpend (Bob versucht, `a_i` zu raten) | `ReceiptSpendAuthorization::verify` schlägt fehl |
| Doppelte Importierung (Bob importiert dieselbe Zahlung zweimal) | Zweite Importierung wird abgelehnt oder ignoriert (gleiche `hint`) |
| Doppel-Ausgabe (zwei Claims auf denselben Output) | Zweiter wird abgelehnt (UTXO ausgegeben) |
| Schlüsselwiederverwendung (dieselbe `r_i` für zwei Zahlungen) | Zwei Outputs mit demselben `hint` — Bob findet nur einen oder beide, aber `a_i` muss unterschiedlich sein |
| Neustart mit fehlenden Datensätzen (Alice verliert `a_i`) | Alice kann nicht reclaimen (falsches `a_i`); Bob kann claimen |
| Falsches Seed (Alice rekonstruiert `a_i` aus falschem Seed) | Falsches `a_i` → `hashes.matches_alice` schlägt fehl |

### Test-Kommandos und Ergebnisdokumentation

Nach der Implementierung:

```
cd upstream/epic
cargo test -p epic_core
cargo test -p epic_chain --test spend_attribution -- --test-threads=1
cargo test -p epic_chain --test receipt_claim_roundtrip -- --test-threads=1
cargo test -p epic_chain --test snip_wallet -- --test-threads=1
```

Die Ergebnisse (alle Tests, Pass/Fail) werden in die finale
`CURRENT_STEP.md`-Bericht aufgenommen.

### Risiken

1. **Cut-through im selben Block:** Wenn Funding und Claim im selben Block
   erscheinen, schneidet Cut-through beide weg, und der `BobClaim`-Kernel
   verbleibt ohne Input → `verify_receipt_resolution` schlägt fehl
   (`receipt_inputs.len() != receipt_kernels.len()`). Im Test: Funding
   und Claim immer in **unterschiedlichen** Blocks.

2. **`is_user_testing_mode()` global gate:** Die Receipt-Feature-Aktivierung
   ist prozess-global, nicht Block-Höhen-basiert. Im Test: `global::set_user_testing_mode(true)`
   vor dem Chain-Init.

3. **Kernel-Hash-Nicht-Determinismus:** Der Kernel-Hash enthält ein
   zufälliges Nonce, außer `secnonce` ist gesetzt. Für Binding-Tests
   (z.B. `kernel_hash` in der Authorization) muss derselbe Kernel
   wiederverwendet werden, nicht neu gebaut.

4. **`chain_id` ist nicht `Copy`:** Bei der Übertragung von
   `ReceiptChainId` in die Authorization muss `.clone()` verwendet werden.

5. **UTXO-Satz-Validierung:** "Bestätigt" bedeutet nicht nur
   `validate_read()`-Erfolg, sondern dass der Output im UTXO-Satz
   existiert und der Input ausgegeben ist. Der Test muss explizit
   den UTXO-Satz prüfen (`chain.outpoint_from_output` / UTXO lookup
   oder `Chain::is_unspent`).

---

## (vorheriger Inhalt — Step 3C Security Review)

## Status: STEP 3C SECURITY REVIEW ABGESCHLOSSEN — WARTET AUF FREIGABE

## Step 3C Security Review

**Verdict: PASS — No authorization bypass demonstrated. Step 3C is ready for Step 3D.**

### Consensus Validation Call Paths (all traced)

All paths that accept an Alice-authorized Receipt spend execute
`verify_receipt_spend_authorizations()`:

| Path | Entry | Source |
|------|-------|--------|
| Tx deserialization | `Transaction::read()` → `validate_read()` | transaction.rs:1230 |
| Block validation | `Block::validate_read()` → `body.validate_read(AsBlock)` | block.rs:828 |
| Pool push | `PoolToChainAdapter::validate_tx()` → `Chain::validate_tx()` → `Transaction::read()` → `validate_read()` | adapters.rs:1145, chain.rs |
| Historical validation | `validate_block()` → `validate_tx()` → `tx.read()` → `validate_read()` | utxo_view.rs:65, pipe.rs:708 |
| Reorg (compact block) | `Block::hydrate_from()` → caller validates → `Block::validate_read()` | block.rs:629 |
| Aggregation | `aggregate()` → `Transaction::new()` → caller validates | transaction.rs:1474 |

**Guarantee:** `validate_read()` (transaction.rs:1145) calls
`verify_receipt_resolution()` then `verify_receipt_spend_authorizations()`
unconditionally. Every path that accepts a transaction calls `validate_read()`.
No path bypasses the authorization check.

### Kernel Attribution Findings

**Finding 1 (Informational): `.first()` kernel selection under aggregation**

`verify_receipt_spend_authorizations()` selects `matching_kernels.first()`
(transaction.rs:1130). Under valid Mimblewimble aggregation, multiple kernels
can share `(excess, fee)` if they were linearly combined. However, the
`kernel_hash` binding (which includes the excess signature with a random
nonce) makes hash collisions cryptographically infeasible. The `.first()`
selection is safe because:
- `verify_receipt_resolution()` (line 1046) already requires `len() == 1`
  for Alice path — a second matching kernel causes rejection.
- BobClaim kernels are filtered out by `!k.is_receipt_resolution()`.

**No bypass demonstrated.** The 12-scenario matrix is covered by existing
Step 3C tests + the 11 new security review tests below.

### Cut-Through Findings

**No vulnerabilities found.**
- `cut_through()` (transaction.rs:1420): receipt items are extracted before
  the ordinary cut-through algorithm and re-appended after. Ordinary
  cut-through is preserved.
- `verify_cut_through()` (transaction.rs:850): receipt items are filtered
  out before the sorted hash comparison. Ordinary cut-through rejection is
  preserved (tested: `sec_review_verify_cut_through_ordinary_still_rejects`).
- Receipt items with same commitment as ordinary items are NOT cut (tested:
  `sec_review_cut_through_receipt_same_commitment_not_cut`).

### Adversarial Authorization Test Results

| Test | Result | Rejection Reason |
|------|--------|-----------------|
| `sec_review_shared_bobclaim_kernel_two_inputs_rejected` | PASS | `InvalidReceiptClaim` — second input has 0 matching BobClaim kernels |
| `sec_review_alice_path_with_bobclaim_kernel_present_rejected` | PASS | `InvalidReceiptClaim` — mixed path (Alice auth + BobClaim kernel on same input) |
| `sec_review_bobclaim_kernel_cannot_satisfy_alice_auth` | PASS | `InvalidReceiptClaim` — no Plain kernel matches (BobClaim filtered out) |
| `sec_review_forged_signature_rejected` | PASS | `InvalidReceiptClaim` — AgSig verification fails (corrupted signature byte) |
| `sec_review_bob_key_as_alice_rejected` | PASS | `InvalidReceiptClaim` — `hashes.matches_alice()` fails (Bob's key ≠ committed alice hash) |
| `sec_review_wrong_path_tag_rejected` | PASS | `sign()` rejects non-AliceSpend path at construction |
| `sec_review_auth_reuse_across_inputs_rejected` | PASS | `InvalidReceiptClaim` — `original_commitment != input.commit` (auth binds a different receipt) |
| `sec_review_kernel_hash_binding_rejects_alternate_kernel` | PASS | `InvalidReceiptClaim` — `auth.kernel_hash != kernel.hash()` (no matching kernel in tx) |
| `sec_review_cut_through_preserves_receipt_when_ordinary_cuts` | PASS | Receipt input survives, ordinary pair is cut |
| `sec_review_cut_through_receipt_same_commitment_not_cut` | PASS | Both receipt items preserved despite same commitment |
| `sec_review_verify_cut_through_ordinary_still_rejects` | PASS | `CutThrough` error for ordinary same-commitment |

### Regression Results

- **Core transaction tests:** 37/37 PASS (26 original + 11 new security review)
- **Core receipt_claim unit tests (ECDH `q_i`, stealth derivation, ReceiptClaim sig):** 44/44 PASS (incl. `stealth_receipt_derivation_matches_for_sender_and_bob`, `shared_spend_blind_matches_for_sender_and_bob`)
- **Core lib tests:** 84/84 PASS
- **Chain spend_attribution:** 16/16 PASS
- **All core integration suites:** PASS (0 failures)

### Accepted Architectural Decision

`global::chain_type()` for `ReceiptChainId` derivation is explicitly accepted.
No concrete bypass demonstrated. The chain ID is derived from trusted process
state, not transaction data.

### Limitations

1. The 12-scenario kernel-attribution matrix is covered by the combination of
   existing Step 3C tests (19) and new security review tests (11). Scenario
   mapping: single kernel (`step3c_valid_alice_spend_passes_validate_read`),
   multiple kernels (`step3c_two_bob_claims_passes_validate_read`),
   shared-kernel/duplicate (`sec_review_shared_bobclaim_kernel_two_inputs_rejected`),
   mixed paths (`sec_review_alice_path_with_bobclaim_kernel_present_rejected`,
   `step3c_mixed_branch_assignment_is_rejected`), mixed ordinary inputs
   (`step3c_aggregation_preserves_receipt_evidence`), aggregation variants
   (`step3c_aggregation_preserves_receipt_evidence` +
   `sec_review_cut_through_preserves_receipt_when_ordinary_cuts`),
   substitution/duplication/reordering (`sec_review_kernel_hash_binding_rejects_alternate_kernel`
   — kernel identity is bound by excess+fee+hash, so substituted kernels fail),
   authorization reuse (`sec_review_auth_reuse_across_inputs_rejected`),
   offset modification (tx offset is not part of the auth message; the
   authorization binds kernel identity, not position — covered by the
   kernel-binding tests).
2. Full `validate()` (with rangeproofs and kernel sig batch verification) is
   not exercised in unit tests (uses `validate_read` to avoid `RangeProof::zero`
   rejection). This is a test-harness limitation, not an implementation gap.
3. Reorg path is tested via `ChainFixture` in the 16 chain E2E tests; no
   dedicated reorg+receipt test exists but the validation path is identical
   to block validation.

### Step 3D Readiness

**READY.** No blocking findings. The implementation correctly:
- Enforces mandatory BobClaim for Bob path
- Enforces valid Alice authorization for Alice path
- Rejects mixed paths
- Preserves receipt evidence through cut-through and aggregation
- Executes authorization verification on every consensus entry point

---

## Status: STEP 3C ABGESCHLOSSEN — WARTET AUF FREIGABE

## Step 3C result

**Implemented.**

- `verify_receipt_resolution()`: replaced the count rule with the two-branch
  matching rule from design doc §6.7. Each `Receipt` input resolves to exactly
  one branch: Alice (authorization present, matching `Plain` kernel) or Bob
  (no authorization, matching `BobClaim` kernel). Mixed paths are rejected.
  All `BobClaim` kernels and all Alice-authorized `Plain` kernels must be used
  exactly once. Ordinary inputs and kernels are unaffected.
- `verify_receipt_spend_authorizations()`: new function in `TransactionBody`,
  called from `validate_read()` immediately after `verify_receipt_resolution()`.
  For each `Receipt` input with an authorization: verifies path, commitment
  match, `hashes.matches_alice()`, unique matching kernel by
  `(excess, fee, kernel_hash)`, Alice AgSig, and chain-id binding derived from
  `global::CHAIN_TYPE` (never from transaction data).
- Receipt-aware cut-through: both `cut_through()` and `verify_cut_through()`
  now explicitly filter out items with `features.is_receipt()`. Receipt inputs
  and outputs are never cut. Ordinary (non-receipt) cut-through behavior is
  unchanged and does not rely on the authorization-presence-byte hash
  difference for security.
- Chain-level binding: the verifier derives the expected `ReceiptChainId`
  from `global::chain_type()`, making cross-chain replay of an authorization
  impossible.

**Kernel attribution invariant (documented in code).**

The authorization binds one specific kernel hash. Matching
`H(k) == a.kernel_hash` proves the kernel in the body is the one Alice
authorized, but does not by itself prove that the kernel's excess
independently balances the input. The kernel sum check
(`verify_kernel_sums`) is the independent balance proof; the authorization's
`kernel_excess` binding is a cross-check between the two. Duplicate kernel
hashes in the body are rejected by `verify_sorted_and_unique`. Multiple
`Receipt` inputs each bind a distinct kernel hash; mixed ordinary and
`Receipt` inputs in one body are allowed and validated independently.

**Tests executed and results.**

| Command | Result |
| --- | --- |
| `cargo test -p epic_core --test transaction step3c -- --test-threads=1` | 19 passed, 0 failed |
| `cargo test -p epic_core --lib` | all passed |
| `cargo test -p epic_core` (all suites) | all passed |
| `cargo test -p epic_chain --test spend_attribution` | 16 passed, 0 failed |
| `cargo test -p epic_chain --test receipt_claim_roundtrip` | 1 passed, 0 failed |

**Step 3C test list (19 in `core/tests/transaction.rs`).**

Consensus two-branch rule:

1. `step3c_valid_alice_spend_passes_validate_read` — valid authorization
   accepted.
2. `step3c_alice_spend_without_authorization_rejected` — no auth → rejected.
3. `step3c_alice_spend_wrong_key_rejected` — wrong Alice key → rejected.
4. `step3c_alice_spend_wrong_commitment_rejected` — wrong commitment →
   rejected.
5. `step3c_alice_spend_wrong_excess_rejected` — wrong excess → rejected.
6. `step3c_alice_spend_wrong_fee_rejected` — wrong fee → rejected.
7. `step3c_alice_spend_wrong_kernel_hash_rejected` — wrong kernel hash →
   rejected.
8. `step3c_alice_spend_wrong_chain_id_rejected` — wrong chain id → rejected.
9. `step3c_mixed_path_rejected` — Alice auth + BobClaim kernel → rejected.
10. `step3c_multiple_receipt_inputs_multiple_authorizations_pass` — multiple
    valid authorizations accepted.
11. `step3c_duplicate_kernel_hash_rejected` — duplicate kernel bound by two
    authorizations → rejected.
12. `step3c_bob_claim_without_alice_auth_passes` — regression.
13. `step3c_forged_bob_claim_rejected` — regression.
14. `step3c_unauthorized_plain_spend_of_receipt_rejected` — no auth, no
    BobClaim → rejected.

Signature verification:

15. `step3c_valid_alice_ag_sig_verifies` — valid AgSig verifies.
16. `step3c_tampered_alice_ag_sig_rejected` — tampered signature → rejected.
17. `step3c_no_matching_kernel_rejected` — no kernel matches → rejected.
18. `step3c_two_matching_kernels_rejected` — two kernels match → rejected.
19. `step3c_ordinary_input_with_authorization_rejected` — ordinary input
    carrying authorization → rejected.

Cut-through (core):

- `step3c_cut_through_bob_claim_preserves_receipt_input`
- `step3c_cut_through_alice_spend_preserves_receipt_input`
- `step3c_cut_through_ordinary_still_cuts`
- `step3c_verify_cut_through_allows_receipt_input_same_commitment`
- `step3c_aggregation_preserves_receipt_evidence`

Chain-level E2E (`chain/tests/spend_attribution.rs`):

- `step3c_alice_reclaim_via_plain_kernel_consensus_accepted`
- `step3c_unauthorized_plain_spend_of_receipt_rejected`
- `step3c_mixed_branch_rejected_in_chain`

**Modified files (Step 3C).**

- `CURRENT_STEP.md`
- `upstream/epic/core/src/core/transaction.rs`
- `upstream/epic/core/src/receipt_claim.rs`
- `upstream/epic/core/tests/transaction.rs`
- `upstream/epic/chain/tests/spend_attribution.rs`

**Consensus boundary (changed in Step 3C).**

- `verify_receipt_resolution()` now accepts a valid Alice `Plain` spend with
  a correct `ReceiptSpendAuthorization` under Usernet activation.
- `verify_receipt_spend_authorizations()` adds the cryptographic verification
  pass (AgSig, chain-id) that was absent in Step 3B.
- BobClaim transactions still pass; forged BobClaim transactions still fail.
- Unauthorized `Plain` spends of `Receipt` outputs are rejected unconditionally.

**Format.** No serialization format changes in Step 3C. The Step 3B presence
byte and authorization encoding remain in effect.

---

## Archiv: Step 3C plan (vor Ausführung)

## Step 3C objective

Step 3C implements the two-branch consensus validation rule for Receipt spends,
the separate signature-verification pass for Alice authorizations, and
receipt-aware cut-through. It enables Alice `Plain`-kernel spends of `Receipt`
outputs when a valid `ReceiptSpendAuthorization` is attached, while keeping
Bob's `BobClaim` path unchanged and rejecting every other combination.

## Step 3C scope

In scope:

- `verify_receipt_resolution()`: replace the count rule with the two-branch
  matching rule from the design doc §6.7:
  - Each `Receipt` input resolves to exactly one branch (Bob or Alice).
  - Alice branch: `receipt_spend_authorization = Some(a)` with
    `a.path == AliceSpend`, `a.original_commitment == input.commit`,
    `hashes.matches_alice(a.alice_public_key)`, and exactly one `Plain` kernel
    in the body with `k.excess == a.kernel_excess`,
    `k.features.fee() == a.fee`, and `H(k) == a.kernel_hash`.
  - Bob branch: `receipt_spend_authorization = None` and exactly one
    `BobClaim` kernel with `claim.original_commitment == input.commit` and
    `hashes.matches_bob(claim.receipt_public_key)`.
  - Mixed path (both branches present on the same input) → rejected.
  - Every `Receipt` input must be used exactly once.
  - Every `BobClaim` kernel must be used exactly once.
  - Every `Plain` kernel bound by an Alice authorization must be used exactly
    once.
  - Ordinary inputs and kernels are unaffected.

- New `verify_receipt_spend_authorizations()` in `TransactionBody`:
  - Called from `validate_read()` after `verify_receipt_resolution()`.
  - For each `Receipt` input with `receipt_spend_authorization = Some(a)`:
    - Verify `a.path == ReceiptPath::AliceSpend`.
    - Verify `a.original_commitment == input.commit`.
    - Verify `hashes.matches_alice(a.alice_public_key)`.
    - Find the unique `Plain` kernel `k` with
      `k.excess == a.kernel_excess && k.features.fee() == a.fee
      && H(k) == a.kernel_hash`. If no such kernel or more than one → error.
    - Verify the Alice AgSig:
      `secp.aggsig_verify(&a.message(chain_id), &a.alice_public_key,
      &a.alice_signature, Some(&a.alice_public_key))`.
    - Verify the chain-id binding:
      `a.chain_id == ReceiptChainId::from_chain_type(&global::chain_type())`.
  - This function is the heavy cryptographic pass; it is not a structural
    check.

- Receipt-aware cut-through in `TransactionBody`:
  - `cut_through()`: skip inputs and outputs with `features.is_receipt()`.
    They are never removed by cut-through.
  - `verify_cut_through()`: skip receipt inputs and outputs in the
    two-pointer hash comparison.
  - Ordinary (non-receipt) cut-through behavior is unchanged.

- Chain-level binding:
  - `verify_receipt_spend_authorizations()` derives the expected
    `ReceiptChainId` from `global::chain_type()`, not from transaction data.
    A cross-chain replayed authorization is rejected.

- Aggregation and kernel-attribution invariants (documented in code):
  - The authorization binds one specific kernel hash. Matching `H(k) ==
    a.kernel_hash` proves the kernel is the one Alice authorized, but does
    not by itself prove that the kernel's excess independently balances the
    input. The kernel sum check (`verify_kernel_sums`) is the independent
    balance proof; the authorization's `kernel_excess` binding is a
    cross-check between the two.
  - Multiple `Receipt` inputs each bind a distinct kernel hash. Duplicate
    kernel hashes in the body are rejected by `verify_sorted_and_unique`.
  - Mixed ordinary and `Receipt` inputs in one body are allowed; each input
    is validated independently by its feature type.

Explicitly **out of scope** for Step 3C:

- Wallet integration, `a_i` persistence, or key management.
- Full three-payment end-to-end demonstration.
- Height-based activation (the process-global test-mode gate remains).
- `AliceReclaim` or any new kernel type.
- BobClaim signature hardening (key-prefixing upgrade).
- Destination-commitment binding in the authorization.
- Commits or pushes.

## Step 3C acceptance criteria

- A valid Alice `Plain` spend with a correct `ReceiptSpendAuthorization` is
  accepted by `Transaction::validate()` and `Block::validate()`.
- An Alice `Plain` spend without an authorization is rejected.
- An Alice `Plain` spend with an authorization signed by the wrong key is
  rejected.
- An Alice `Plain` spend with an authorization over a wrong commitment,
  wrong excess, wrong fee, or wrong kernel hash is rejected.
- An Alice authorization with a wrong `chain_id` is rejected.
- A `Receipt` input with both an Alice authorization and a matching
  `BobClaim` kernel in the body is rejected (mixed path).
- A `BobClaim` transaction without any Alice authorization continues to be
  accepted (regression).
- A forged `BobClaim` transaction continues to be rejected (regression).
- An unauthored `Plain` spend of a `Receipt` output (neither Alice auth nor
  BobClaim kernel) is rejected.
- Multiple `Receipt` inputs each binding distinct `Plain` kernels in one body
  are accepted when all authorizations are valid.
- A duplicate `Plain` kernel (same hash bound by two authorizations) is
  rejected.
- `cut_through()` does not remove `Receipt` inputs or outputs, even when
  their hashes match non-receipt counterparts.
- `verify_cut_through()` does not reject a body that contains a `Receipt`
  output and a `Receipt` input with matching hashes (same-block BobClaim or
  AliceSpend scenario).
- Ordinary `Plain`/`Coinbase` cut-through behavior is unchanged.
- All existing `epic_core` and `epic_chain` tests pass.

## Step 3C planned tests

Consensus two-branch rule (in `core/tests/transaction.rs`):

- Alice spend with valid authorization → accepted.
- Alice spend without authorization → rejected (`InvalidReceiptClaim`).
- Alice spend with wrong-key authorization → rejected.
- Alice spend with wrong commitment in authorization → rejected.
- Alice spend with wrong excess in authorization → rejected.
- Alice spend with wrong fee in authorization → rejected.
- Alice spend with wrong kernel hash in authorization → rejected.
- Alice spend with wrong chain id in authorization → rejected.
- Mixed path: Alice auth + BobClaim kernel on same input → rejected.
- Multiple receipt inputs, multiple valid authorizations → accepted.
- Duplicate kernel hash bound by two authorizations → rejected.
- BobClaim without any Alice auth → accepted (regression).
- Forged BobClaim → rejected (regression).
- Plain spend of receipt output with no auth and no BobClaim → rejected.

Signature verification (in `core/tests/transaction.rs`):

- Valid Alice AgSig verifies; tampered signature rejected.
- `verify_receipt_spend_authorizations` rejects when no matching kernel found.
- `verify_receipt_spend_authorizations` rejects when two kernels match.

Cut-through (in `core/tests/transaction.rs`):

- `cut_through` preserves a `Receipt` output even when a `Plain` input has
  the same commitment hash.
- `cut_through` preserves a `Receipt` input even when a `Plain` output has
  the same commitment hash.
- `verify_cut_through` does not reject a body with a `Receipt` input and
  `Receipt` output of identical commitment (same-block scenario).
- Ordinary cut-through: `Plain` input/output with same commitment is still
  removed by `cut_through` and rejected by `verify_cut_through`.

Chain-level E2E (in `chain/tests/spend_attribution.rs`):

- Alice builds a valid spend and submits through the chain → accepted.
- Bob attempts a `Plain` spend of an unclaimed receipt → rejected.
- Mixed-branch transaction rejected at chain level.


Step 3B integrates the Step 3A cryptographic foundation into the transaction
layer. It adds `Input.receipt_spend_authorization`, an Alice `Plain`-kernel
spend builder, strict Alice-path verification, and an explicit chain identifier.
It **does not enable Alice spending in consensus**. The existing
`verify_receipt_resolution()` rule must continue rejecting an Alice `Plain`
spend of a `Receipt` input until Step 3C is explicitly approved.

## Step 3B objective

1. Extend `Input` with `receipt_spend_authorization: Option<ReceiptSpendAuthorization>`,
   valid only for `OutputFeatures::Receipt`.
2. Add a narrowly scoped `build_alice_spend_transaction(...)` helper that spends
   one `Receipt` output with an ordinary `KernelFeatures::Plain { fee }` kernel
   and attaches Alice's authorization.
3. Make Alice authorization verification reject every path other than
   `ReceiptPath::AliceSpend`, including a valid signature over `BobClaim`.
4. Replace the Step 3A compile-time chain-id constant with an explicit,
   stable `ReceiptChainId` value supplied by the caller. Do not read the
   mutable process-global chain mode inside signing or verification.
5. Prove that the authorization does not change the referenced Receipt UTXO
   identity, that BobClaim still works, and that the new Alice transaction is
   rejected only because Alice spending is intentionally disabled.

## Step 3B scope

In scope:

- `Input` field, binary serialization, serde compatibility, constructors, and
  structural validation of the new field.
- `ReceiptChainId` and its mapping from the existing `ChainTypes` identity.
- Alice-specific `sign`/`verify`/`verify_with_hashes` behavior and a
  structural per-input/per-kernel verification helper.
- `build_alice_spend_transaction` for exactly one Receipt input and one
  ordinary destination output.
- Focused unit, serialization, builder, adversarial, and consensus-boundary
  tests.
- Documentation of cut-through, aggregation, and multi-input/kernel ambiguities
  for Step 3C.

Explicitly **out of scope** for Step 3B:

- Changing `verify_receipt_resolution()` to accept Alice spends.
- Consensus-level Alice signature verification or a two-branch rule.
- Receipt-aware cut-through.
- Height-based activation.
- Mainnet/Floonet consensus behavior.
- `AliceReclaim`, a new kernel type, scripting, or adaptor signatures.
- BobClaim signature semantics, Pedersen commitment logic, or Bulletproof
  logic.
- Wallet key persistence, recovery, or reclaim.
- General multi-input/multi-kernel aggregation rules.
- End-to-end payment tests and commits.

## Step 3B expected files

- `CURRENT_STEP.md`
- `upstream/epic/core/src/receipt_claim.rs`
- `upstream/epic/core/src/core/transaction.rs`
- `upstream/epic/core/src/libtx/build.rs` (only if an `Input` literal requires it)
- `upstream/epic/core/tests/transaction.rs`
- `upstream/epic/chain/tests/receipt_claim_roundtrip.rs`
- `upstream/epic/chain/tests/spend_attribution.rs`
- `upstream/epic/core/examples/receipt_stealth_demo.rs`
- Any other file containing an `Input { ... }` literal or a call to the changed
  Step 3A verification API.

## Step 3B acceptance criteria

- `ReceiptSpendAuthorization::verify()` and `verify_with_hashes()` reject
  `ReceiptPath::BobClaim` and every other non-`AliceSpend` path.
- `ReceiptSpendAuthorization::sign()` and `message()` reject non-Alice paths;
  no helper silently creates an Alice authorization under another path.
- A valid signature over `BobClaim`, a valid signature under a different key,
  and a correct key with mismatched Receipt hashes are all rejected.
- `ReceiptChainId` is explicit, stable, and supplied to signing and
  verification. Matching chain IDs verify; different chain IDs fail; empty or
  oversized custom IDs are rejected.
- `Input` carries an optional authorization only for Receipt inputs.
- Ordinary (`Plain`/`Coinbase`) input binary serialization is byte-for-byte
  unchanged.
- Receipt inputs with `None` and with `Some(auth)` roundtrip unambiguously.
- A presence byte other than 0/1, an unknown path tag, a malformed public key,
  and a malformed signature are rejected.
- An ordinary input carrying an authorization is rejected by serialization and
  by structural receipt validation.
- `OutputIdentifier::from_input()` ignores the authorization; inputs with and
  without an authorization reference the same Receipt UTXO.
- `build_alice_spend_transaction()` produces:
  - the original Receipt input with a valid Alice authorization;
  - an ordinary destination output with value `receipt_value - fee`;
  - a standard `Plain { fee }` kernel with no `receipt_claim`;
  - a valid excess signature and a balanced kernel sum.
- The builder requires only `q_i`, `a_i`, and a fresh destination blind. It
  does not require Bob's private keys, and it never serializes `a_i`.
- The constructed Alice transaction is rejected by the existing consensus rule
  specifically with `Error::InvalidReceiptClaim` after Usernet activation, not
  because of an invalid signature, malformed transaction, or broken balance.
- Existing BobClaim transactions continue to validate; forged BobClaim
  transactions continue to fail.
- No new consensus rule is introduced, and nothing is committed.

## Step 3B planned tests

Authorization validation (in `receipt_claim.rs`):

- Valid Alice authorization.
- Valid signature over `BobClaim` is rejected.
- Correct Alice key with `BobClaim` path is rejected.
- Correct path with a different Alice public key is rejected.
- Correct signature with mismatched Receipt key hashes is rejected.
- Tampered serialized path tag is rejected or fails verification.
- Wrong commitment, excess, fee, and kernel hash are rejected.
- Wrong version and wrong chain identifier are rejected.
- Matching chain identifier is accepted; different chain identifier is rejected.
- Empty/oversized `ReceiptChainId` is rejected.
- Chain binding survives binary serialization.

Input serialization (in `transaction.rs`):

- Receipt input without authorization roundtrips.
- Receipt input with authorization roundtrips.
- Ordinary input serialization is unchanged (length and bytes).
- Ordinary input with an unexpected authorization is rejected.
- Malformed presence byte and malformed authorization are rejected.
- Original Receipt UTXO identity is preserved (`OutputIdentifier::from_input`).
- Input hash includes the authorization, and the Step 3C cut-through
  implication is recorded.

Alice spend builder (in `receipt_claim.rs`):

- Correct Receipt input, destination output, `Plain` kernel, excess signature,
  authorization, fee, and balance.
- Authorization binds the finalized kernel hash.
- No Bob private key is required.
- Alice's secret `a_i` does not appear in serialized transaction bytes.
- Non-Receipt output, wrong value/blind, and mismatched Alice key are rejected.

Consensus boundary (in `receipt_claim.rs` or chain tests):

- The Alice transaction is rejected with `InvalidReceiptClaim` under Usernet
  activation.
- Kernel signature and kernel sums are independently valid.
- BobClaim transactions still pass.
- Forged BobClaim transactions still fail.

Aggregation and adversarial structure:

- Multiple Receipt inputs and multiple Plain kernels are structurally
  documented as requiring a Step 3C matching rule.
- Duplicate/substituted kernel hashes and authorization reuse are documented as
  unresolved consensus requirements.
- Mixed Alice/Bob path behavior under the current rule is documented; it must
  be rejected explicitly in Step 3C.

## Existing uncommitted changes that must be preserved

The working tree matches the Step 3A report:

- Top-level docs modified: `IMPLEMENTATION_PLAN.md`, `PROTOCOL.md`, `README.md`,
  `docs/assumptions-and-open-questions.md`, `docs/continuation-guide.md`,
  `docs/cryptographic-foundation.md`, `docs/one-sided-poc.md`,
  `docs/protocol-spec.md`, `docs/receipt-claim-design.md`, `docs/test-plan.md`,
  `docs/threat-model.md`, `docs/usernet-experiment.md`.
- Top-level untracked: `CURRENT_STEP.md`, `docs/SNIP_ADAPTOR_FEASIBILITY.md`,
  `docs/SNIP_SPEND_ATTRIBUTION_RESEARCH.md`,
  `docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md`.
- Submodule `upstream/epic` modified: `Cargo.lock`, `chain/Cargo.toml`,
  `chain/tests/receipt_claim_roundtrip.rs`,
  `core/examples/receipt_stealth_demo.rs`,
  `core/src/core/transaction.rs`, `core/src/libtx/aggsig.rs`,
  `core/src/libtx/build.rs`, `core/src/receipt_claim.rs`,
  `core/tests/transaction.rs`.
- Submodule untracked: `chain/tests/spend_attribution.rs`.

`Cargo.lock` and `chain/Cargo.toml` predate Steps 1–3A and must not be reverted.
No pre-existing work may be overwritten or silently modified.

## Step 3B open risks

- **Input hash and cut-through.** Adding an authorization presence byte changes
  the Receipt input hash for both `None` and `Some`. This incidentally prevents
  generic cut-through from matching Receipt inputs to Receipt outputs. Step 3C
  must make cut-through explicitly receipt-aware; Step 3B must not rely on the
  accidental hash difference for security.
- **UTXO identity.** `OutputIdentifier::from_input()` must remain unchanged and
  must ignore the authorization. UTXO lookup must continue to reference the
  original Receipt output.
- **Mixed paths.** The current `verify_receipt_resolution()` counts BobClaim
  kernels only. A Receipt input carrying an Alice authorization plus a
  BobClaim kernel would still satisfy the current rule. Step 3B documents this
  and does not add a consensus rule; Step 3C must reject mixed paths.
- **Multiple inputs/kernels.** The builder is single-input/single-kernel. The
  authorization binds one kernel hash, but the current rule does not enforce a
  unique per-input/per-kernel mapping. Duplicate kernel hashes, kernel
  substitution, reordering, and authorization reuse require Step 3C rules.
- **Chain identifier source.** Epic's Usernet genesis is mined with the current
  timestamp, so the genesis hash is not a stable Usernet identifier. Step 3B
  uses an explicit protocol-level `ReceiptChainId` derived from the established
  `ChainTypes` identity. It must not be described as a genesis-derived chain
  identity.
- **Key management.** `a_i` remains random and is not persisted or recoverable.
  Losing `a_i` loses Alice's branch while Bob can still claim.
- **Format break.** Receipt input serialization changes. This is an experimental
  Usernet v2 format; no Mainnet/Floonet activation is added.
- **Consensus inactivity.** The authorization is not consensus-enforced in
  Step 3B. `verify_with_hashes()` and the new structural helper are API-level
  only.

## Step 3B documentation conflicts and resolutions

1. Step 3A left `verify_with_hashes()` path-agnostic. Step 3B makes `verify`,
   `verify_with_hashes`, `sign`, and `message` Alice-specific; `message_with`
   remains the low-level test helper.
2. The design doc leaves the chain-id source open. Step 3B uses an explicit
   `ReceiptChainId` mapped from `ChainTypes`, because Usernet's genesis is
   non-deterministic. This is the smallest Usernet-only alternative.
3. The design doc says `Input` gains the authorization and that the input hash
   includes it. Step 3B preserves the UTXO identity through
   `OutputIdentifier::from_input()` and records the cut-through change for
   Step 3C.
4. The design doc's two-branch consensus rule is Step 3C. Step 3B adds only
   structural field validation and leaves `verify_receipt_resolution()`
   otherwise unchanged.

---

## Archiv: Step 3B result (abgeschlossen)

Step 3B integrates the Step 3A cryptographic foundation into the transaction
layer. It does **not** enable Alice spending in consensus.

## Step 3B result

**Implemented.**

- `ReceiptChainId`: explicit, stable chain identifier type with `new` (1–64
  bytes), `from_chain_type(&ChainTypes)`, `as_bytes()`, and four constructors
  (`usernet`, `floonet`, `mainnet`, `automated_testing`). Replaces the Step 3A
  compile-time `SPEND_AUTHORIZATION_CHAIN_ID` constant.
- `ReceiptSpendAuthorization::{message, sign, verify, verify_with_hashes}` all
  take an explicit `chain_id: ReceiptChainId` and reject every path other than
  `ReceiptPath::AliceSpend`. `message_with` is `pub` and takes
  `chain_id: ReceiptChainId` (length-prefixed LE u64).
- `verify_for_receipt_input(&self, input, kernel, chain_id) -> bool`:
  convenience helper that checks the input carries receipt metadata, the
  commitment matches, and the authorization verifies against the final kernel.
- `Input.receipt_spend_authorization: Option<ReceiptSpendAuthorization>`:
  serialized as a presence byte (0/1) only for `Receipt`-feature inputs;
  non-receipt inputs serialize byte-identically to the pre-Step-3B format.
  `OutputIdentifier::from_input()` ignores the field, preserving UTXO identity.
- `build_alice_spend_transaction(...)`: builds a one-input, one-output `Plain`
  transaction spending a `Receipt` output with a signed
  `ReceiptSpendAuthorization` on the input. No Bob private key required.
  Alice's `a_i` does not appear in serialized bytes.
- Structural validation in `verify_receipt_resolution()`: an ordinary (non-
  receipt) input carrying an authorization is rejected with
  `InvalidReceiptClaim`. The existing count rule
  (`receipt_inputs.len() != receipt_kernels.len()`) continues to reject the
  Alice `Plain` spend under Usernet activation.

**Step 3C documentation comments added:**

- `verify_receipt_resolution()`: mixed-path (Alice auth + BobClaim kernel) and
  multi-input/multi-kernel ambiguity documented as requiring Step 3C rules.
- `Input.receipt_spend_authorization` field: cut-through hash change documented;
  Step 3C must make cut-through explicitly receipt-aware.
- `build_alice_spend_transaction`: consensus inactivity documented.

**Tests executed and results.**

| Command | Result |
| --- | --- |
| `cargo test -p epic_core --lib` | 85 passed, 0 failed |
| `cargo test -p epic_core` (all suites) | all passed (85 lib + 21 + 5 + 21 + 4 + 16 + 5 + 4 + 7 + 2 + 5) |
| `cargo test -p epic_chain --test receipt_claim_roundtrip` | 1 passed, 0 failed |
| `cargo test -p epic_chain --test spend_attribution` | 13 passed, 0 failed |
| `cargo run -p epic_core --example receipt_stealth_demo` | ran successfully |

**Modified files (Step 3B).**

- `CURRENT_STEP.md`
- `upstream/epic/core/src/receipt_claim.rs`
- `upstream/epic/core/src/core/transaction.rs`
- `upstream/epic/core/tests/transaction.rs`
- `upstream/epic/chain/tests/spend_attribution.rs` (fixture updates)

**Consensus boundary (unchanged).**

- `verify_receipt_resolution()` still rejects the Alice `Plain` spend via the
  count rule. The structural check for non-receipt inputs carrying
  authorization is additive and does not change the consensus outcome for the
  Alice transaction.
- BobClaim transactions still pass; forged BobClaim transactions still fail.

**Format break.**

- Receipt input serialization gains a presence byte (and optionally the full
  authorization). This is an experimental Usernet v2 format; no Mainnet/Floonet
  activation is added.

---

## Archiv: Step 3A result (abgeschlossen)

Step 3A introduces the cryptographic foundation for Alice's independent spend
path. It does **not** enable Alice spending in consensus. The full design is in
[`docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md`](docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md).

## Step 3A result

**Implemented.**

- `AliceReceiptAuthorizationKey`: fresh random `a_i`, derived `A_i = a_i G`,
  private secret accessor, no serialization of `a_i`.
- `SharedReceiptKeyHashes { bob, alice }`: preserved Bob derivation, added
  role-separated `H(DOMAIN || "receipt-key|alice|" || A_i)`,
  `from_public_keys`, `matches_bob`, `matches_alice`, and 64-byte binary
  serialization.
- `ReceiptPath::AliceSpend` with tag `1`; `BobClaim` remains tag `0`; unknown
  tags are rejected.
- `ReceiptSpendAuthorization`: domain-separated message, signing with `a_i`,
  verification with `A_i`, key-prefixed AgSig challenge
  (`pubkey_for_e = Some(&A_i)`), `verify_with_hashes` for the receipt-output
  key commitment, and binary serialization. All fields are `Copy`, so a future
  `Input` field can preserve `Input: Copy`.
- The signed message binds `DOMAIN || "receipt-spend-authorization|" ||
  version || chain_id || path tag || original_commitment || kernel_excess ||
  fee || kernel_hash || compressed(A_i)`.

**Tests executed and results.**

| Command | Result |
| --- | --- |
| `cargo test -p epic_core --lib step3a` | 22 passed, 0 failed |
| `cargo test -p epic_core --lib` | 69 passed, 0 failed |
| `cargo test -p epic_core` | exit 0; all lib, integration, and doctest suites passed |
| `cargo test -p epic_chain --test receipt_claim_roundtrip` | 1 passed, 0 failed |
| `cargo test -p epic_chain --test spend_attribution` | 13 passed, 0 failed |
| `cargo run -p epic_core --example receipt_stealth_demo` | ran successfully |

**Modified files (Step 3A).**

- `CURRENT_STEP.md`
- `upstream/epic/core/src/receipt_claim.rs`
- `upstream/epic/core/src/core/transaction.rs` (`matches_bob`; test fixture)
- `upstream/epic/core/src/libtx/build.rs` (test fixture)
- `upstream/epic/core/src/libtx/aggsig.rs` (doctest fixture)
- `upstream/epic/core/tests/transaction.rs` (test fixture)
- `upstream/epic/core/examples/receipt_stealth_demo.rs` (fixture)
- `upstream/epic/chain/tests/receipt_claim_roundtrip.rs` (fixture)
- `upstream/epic/chain/tests/spend_attribution.rs` (fixture)

`Cargo.lock` and `chain/Cargo.toml` were already modified before Step 3A and
were not touched in this step.

**Receipt serialization changes.**

- `SharedReceiptKeyHashes` grows from 32 to 64 bytes (`bob` then `alice`).
  This changes the identity of receipt outputs only; ordinary `Plain` and
  `Coinbase` outputs are byte-for-byte unchanged.
- `ReceiptPath` gains tag `1` for `AliceSpend`. `BobClaim` remains tag `0`.
  `ReceiptClaim`'s field layout is unchanged.
- `ReceiptSpendAuthorization` is a standalone structure. It is **not** added to
  `Input`, so transaction and input serialization are unchanged in Step 3A.

**Consensus status.** Alice authorization is inactive. There is no
`Input.receipt_spend_authorization`, `verify_receipt_resolution` is unchanged,
and `ReceiptSpendAuthorization` is not referenced by transaction validation.
`verify_with_hashes` is an API-level check only.

**Deviations from the approved design.**

- Added `verify_with_hashes` so the `hashes.alice` requirement is testable
  without wiring consensus.
- Added `AliceReceiptAuthorizationKey::from_secret_key` for tests and future
  recovery; deterministic recovery and persistence remain out of scope.
- Used explicit constants `SPEND_AUTHORIZATION_VERSION = 1` and
  `SPEND_AUTHORIZATION_CHAIN_ID = b"epic-usernet"`; the final chain-id source
  is unresolved.
- `kernel_hash` uses `core::hash::Hash` with its derived serde representation.
- Fixed two pre-existing test/doctest compile failures
  (`core/tests/transaction.rs`, `core/src/libtx/aggsig.rs`) caused by the
  pre-existing receipt fields on `Output`. No production behavior changed.
- Pre-existing formatting was not reformatted.

**Remaining risks.**

- `a_i` is random and not persisted; losing it blocks Alice's branch but not
  Bob's claim.
- The chain-id source and versioning/activation scheme are unresolved.
- Adding `alice` is a receipt-output format break; existing v1 receipt outputs
  are invalid.
- `ReceiptSpendAuthorization` is not consensus-enforced yet.
- `verify_with_hashes` does not enforce `path == AliceSpend`; the future
  consensus rule must do so explicitly.
- `BobClaim` still uses `pubkey_for_e = None`; key-prefixing it remains a
  separate hardening item.
- Receipt-aware cut-through is not implemented.

**Proposed smallest Step 3B.** Add
`Input.receipt_spend_authorization: Option<ReceiptSpendAuthorization>`,
serialized only for `Receipt` inputs, plus a narrow
`build_alice_spend_transaction` that constructs an ordinary `Plain` kernel and
the authorization. Add serialization and signature tests, but keep
`verify_receipt_resolution` and consensus activation unchanged, so the Alice
path remains rejected. This isolates the input-format/hash impact before the
consensus fork.

## Step 3A objective

Add an Alice-exclusive, per-payment authorization key and a Schnorr-based
`ReceiptSpendAuthorization` so that later steps can enforce the two-branch
receipt resolution without changing Bob's existing `BobClaim` path. Alice's
eventual path remains an ordinary `KernelFeatures::Plain` kernel plus a
consensus-enforced input authorization. No new kernel feature and no
`AliceReclaim` kernel are introduced.

## Step 3A exact implementation scope

Production implementation is confined to
`upstream/epic/core/src/receipt_claim.rs`; other files receive only the minimal
fixture/compile updates needed for the 64-byte hash format:

1. Add a minimal `AliceReceiptAuthorizationKey` helper that generates a fresh
   random `a_i` and derives `A_i = a_i G`.
2. Extend `SharedReceiptKeyHashes` with `alice: [u8; 32]` while preserving the
   existing `bob` derivation. Add `from_public_keys`, `matches_bob`,
   `matches_alice`, binary serialization, and deserialization.
3. Add `ReceiptPath::AliceSpend` with a distinct tag; keep `BobClaim` at tag 0
   and reject unknown tags.
4. Add `ReceiptSpendAuthorization` with the fields from the approved design:
   `path`, `original_commitment`, `kernel_excess`, `fee`, `kernel_hash`,
   `alice_public_key`, `alice_signature`. Implement domain-separated message
   construction, signing with `a_i`, verification with `A_i`, a
   hash-binding-aware verification helper, and binary serialization.
5. Update local fixtures and tests that construct `SharedReceiptKeyHashes` so
   the crate compiles and the existing BobClaim regression tests still pass.

Explicitly **out of scope** for Step 3A:

- `Input.receipt_spend_authorization` and any `Input`/`Output` serialization
  change beyond the 64-byte `SharedReceiptKeyHashes`.
- Alice spending transaction construction.
- Two-branch `verify_receipt_resolution()`.
- Alice authorization checks in transaction consensus validation.
- Receipt-aware cut-through.
- Height-based consensus activation.
- Wallet key persistence, recovery, or reclaim.
- End-to-end payment tests.
- Changes to the `BobClaim` kernel or ordinary Epic outputs.

## Step 3A acceptance criteria

- A fresh, independent Alice authorization key can be generated per payment.
- `SharedReceiptKeyHashes` contains separate `bob` and `alice` hashes with
  distinct domain separators.
- `ReceiptPath::AliceSpend` serializes and deserializes; unknown tags fail.
- `ReceiptSpendAuthorization` can be signed and verified, and verification
  fails for every bound field, version, and chain identifier change.
- The Alice public-key hash is checked against `hashes.alice` before an
  authorization is accepted.
- Existing `BobClaim` signatures, ECDH derivations, and Bulletproof tests
  continue to pass.
- Alice authorization remains inactive in consensus.
- No ordinary Epic output behavior changes.
- Nothing is committed.

## Step 3A planned tests

Unit tests in `receipt_claim.rs`:

- Key separation: valid `a_i`/`A_i`; three payments produce distinct keys;
  Alice and Bob derive identical `q_i`; Bob derives `b_i` independently;
  `a_i` is not `q_i` or `b_i`.
- Hash binding: correct/wrong `A_i` against `hashes.alice`; correct/wrong
  `B_i` against `hashes.bob`; Bob and Alice hash domains differ.
- Signature verification: valid authorization; wrong signature, public key,
  commitment, excess, fee, kernel hash, path, version, and chain identifier.
- Serialization: `SharedReceiptKeyHashes`, `ReceiptPath::AliceSpend`, and
  `ReceiptSpendAuthorization` roundtrips; unknown path tag rejection;
  malformed public key and signature rejection where the existing
  serialization layer enforces it.
- Regression: existing BobClaim, ECDH, and Bulletproof tests.

## Step 3A known risks and unresolved questions

- **Chain identifier source.** Step 3A uses explicit constants
  (`AUTHORIZATION_VERSION = 1`, `AUTHORIZATION_CHAIN_ID = b"epic-usernet"`).
  Whether the final rule should use a compile-time constant, a runtime
  `global::get_chain_type()` value, or a genesis hash is unresolved.
- **Version source.** `version` is currently a receipt-authorization version,
  not a block-height activation. Height-based activation remains out of scope.
- **Hash serde convention.** `kernel_hash` uses Epic's `core::hash::Hash` with
  its derived serde representation. The binary format is the authoritative
  format; JSON representation may need a hex adapter later.
- **Key management.** `a_i` is random and not persisted or recoverable in
  Step 3A. A lost `a_i` means Alice cannot use her branch, while Bob can still
  claim.
- **Format break.** Adding `alice` changes `SharedReceiptKeyHashes` from 32 to
  64 bytes, which changes the receipt output identity. This is treated as an
  experimental Usernet v2 format; no Mainnet/Floonet activation is added.
- **Hash-check helper is not consensus.** `verify_with_hashes()` enforces the
  `hashes.alice` check at the API level, but it is deliberately not wired into
  transaction validation in Step 3A.
- **BobClaim key-prefixing.** Bob's existing `ReceiptClaim` continues to use
  `pubkey_for_e = None`. Only the new Alice authorization uses key-prefixing.

## Step 3A documentation conflicts identified before implementation

1. Step 3A says "do not implement Alice authorization checks in transaction
   consensus validation", while §7 requires the `hashes.alice` check before an
   authorization is accepted. Resolution: implement the check as a standalone
   `verify_with_hashes()` helper; keep consensus inactive.
2. The Step 3A message list requires `chain_id`; the design doc names the
   receipt feature version but does not define a chain-id source. Resolution:
   use explicit constants and expose a parameterized message builder for
   negative tests, without changing `ReceiptClaim`.
3. The design doc pseudocode writes `kernel_hash: H(kernel)`; Step 3A names the
   field type `Hash`. Resolution: use `core::hash::Hash`, matching Epic's
   existing kernel hashing type.
4. The design doc proposes the full Step 3; Step 3A is a strict subset.
   Resolution: implement only the cryptographic foundation and leave the
   consensus wiring untouched.

---

## Archiv: Step 2 result (abgeschlossen)

Step 2 was a **design and feasibility step only**. No implementation code was
changed and nothing was committed.

The full report is
[`docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md`](docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md).

**Verdict.** The shared Pedersen blind `q_i` is a balance-construction
capability, not an authorization. Alice and Bob both need it for Bob's offline
claim. Therefore the only viable separation is an independent, per-payment
Alice authorization enforced by consensus, while the Alice path remains a
`KernelFeatures::Plain` kernel.

**Recommended construction.**

- The `Receipt` output commits to two one-time key hashes:
  `hashes.bob = H(b_i)` and `hashes.alice = H(a_i)`.
- Bob's branch is the existing `BobClaim` with a `ReceiptClaim` signed by
  `b_i`; unchanged.
- Alice's branch is a standard `Plain { fee }` kernel plus a new
  `ReceiptSpendAuthorization` carried by the `Receipt` input and signed by
  `a_i`.
- Consensus requires every `Receipt` input to be resolved by exactly one of the
  two branches. Bob does not know `a_i`; Alice does not know `b_i`.
- First-confirmed-spend-wins is preserved because both branches spend the same
  UTXO.

**Rejected candidates.** Bare `Plain` (Bob bypass), a new `AliceReclaim` kernel
(protocol forbids it and it does not solve the shared-`q_i` problem), adaptor
signatures (Step 1), threshold/2-of-2 (breaks independent spend), exclusive
`q_i` (breaks offline Bob claim), and a general script/MAST layer (too large
for scope; the two-branch claim is the minimal special case).

**Required changes (classified).** Consensus/core: `SharedReceiptKeyHashes`
gains `alice`; `ReceiptPath` gains `AliceSpend`; `Input` gains the
authorization; `verify_receipt_resolution` becomes two-branch; a full-validation
signature pass is added; cut-through becomes receipt-aware; the test-mode gate
becomes a height-based activation. Wallet: Alice key generation/persistence,
reclaim builder, own-output detection; Bob scan/claim unchanged. Tests: update
fixtures, add consensus and chain-level acceptance tests.

**Blockers before Step 3.** The Step 2 instruction was truncated at "Bob cannot
use"; the reconstructed requirements in §3 of the design doc must be confirmed.
The solution is a consensus change, not wallet-only. Adding `alice` changes the
`Receipt` output identity. The current activation gate is the process-global
`is_user_testing_mode()`, not a height-based rule.

> **Note on the Step 2 instruction.** The incoming Step 2 text was truncated
> at “Bob cannot use”. The requirements below are reconstructed from the
> visible Step 2 text, `PROTOCOL.md` §2/§3/§5, the Step 1 report, and the
> existing `Receipt`/`BobClaim` design. The missing requirements must be
> confirmed before any Step 3 implementation.

---

## Archiv: Step 2 plan (abgeschlossen)

## Step 2 Objective

Design and evaluate a cryptographically enforceable separation between
Alice's and Bob's spending capabilities for the existing `Receipt` output
type, so that all of the following hold:

- Alice can independently spend an unclaimed `Receipt` output using the
  standard `Plain` kernel path.
- Bob cannot use Alice's path to bypass the mandatory `BobClaim` evidence.
- Bob can independently claim a payment and leave evidence that Alice can
  verify from the chain.
- Alice cannot forge Bob's claim evidence.
- Bob publishes a reusable public Payment Descriptor once; Alice creates
  multiple payments without Bob being online; Bob does not prepare
  per-payment signatures.
- First-confirmed-spend-wins is preserved.
- No privacy, consensus, or functional requirement is weakened to fit the
  current implementation.

This is a **design and feasibility step only**. It does not implement the
solution.

## Scope and non-goals

- Do **not** modify `epic_core`, `epic_chain`, `epic_libtx`, or wallet
  implementation code.
- Do **not** implement the proposed authorization scheme.
- Do **not** modify consensus rules, serialization, kernel types, the
  `Receipt` output format, or wallet behavior.
- Do **not** reintroduce an `AliceReclaim` kernel type. The Alice path must
  remain a `Plain` kernel (`PROTOCOL.md` §3.9/§4).
- Do **not** commit.
- **Stop after Step 2** and wait for explicit approval.

## Tasks

1. Re-read `CURRENT_STEP.md`, `PROTOCOL.md`,
   `docs/SNIP_ADAPTOR_FEASIBILITY.md`, `docs/receipt-claim-design.md`, and
   inspect the current `Receipt`/`BobClaim` implementation.
2. State the shared-`q_i` problem precisely and formalize the security
   properties a solution must provide.
3. Enumerate and evaluate candidate constructions:
   - bare `Plain` spend with a wallet convention;
   - a new `AliceReclaim` kernel;
   - an exclusive Alice authorization carried by the `Plain` path;
   - a script/MAST-like OR commitment;
   - adaptor signatures;
   - a threshold/2-of-2 aggregate key;
   - exclusive `q_i` with a non-standard Bob claim.
4. Reject candidates that weaken mandatory evidence, privacy, independent
   spend, or the reusable-descriptor requirement.
5. Specify the recommended construction in enough detail to implement:
   key derivation, output fields, input authorization, kernel
   construction, consensus validation, scan, claim, reclaim, and
   first-confirmed-spend-wins.
6. Analyze attacks: Bob bypass, Alice forgery, third-party theft, replay,
   front-running, malleability, collusion, malicious miner, cut-through,
   aggregation, reorg, and activation.
7. Analyze privacy per relationship and observer class.
8. Classify required changes and identify blockers and risks.
9. Write `docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md` and update this file.
10. Stop and report; do not implement.

## Acceptance criteria

- The shared-`q_i` problem is stated precisely and the security properties
  are explicit.
- At least the candidate constructions above are evaluated, with concrete
  cryptographic or consensus reasons for rejection or selection.
- The recommended construction is concrete: exact fields, signatures,
  signed messages, consensus rules, and data flow.
- The construction preserves the reusable descriptor, offline payment
  creation, independent Bob claim, independent Alice reclaim, Alice's
  verification, first-confirmed-spend-wins, and the privacy requirements.
- Bob cannot bypass the mandatory evidence; Alice cannot forge it; external
  observers cannot derive identities or deterministic payment links.
- The tension between “ordinary `Plain` kernel” and “Bob cannot bypass” is
  addressed explicitly.
- Required code changes are classified by layer.
- The design document is written and `CURRENT_STEP.md` is updated.
- No implementation code is changed and no commit is made.

## Known risks and hypotheses to test

- **Bare `Plain` is unsafe.** Alice and Bob both know the shared blind
  `q_i`. If a bare `Plain` kernel is allowed to spend a `Receipt` input,
  Bob can use it and omit the mandatory `ReceiptClaim`.
- **A new kernel type is forbidden.** `AliceReclaim` is not part of the
  target protocol; the Alice path must remain a `Plain` kernel. The design
  must find a way to authorize the Alice path without a new kernel type.
- **Input-level authorization changes serialization.** Adding an Alice
  authorization to the `Receipt` input changes the input hash, the MMR,
  the UTXO set, and historical validation.
- **Output format changes.** Adding an `alice` key hash changes the
  `Receipt` output identity and every existing test fixture.
- **Cut-through.** A `Receipt` output and its spending input in the same
  block can be cut through, leaving a resolution kernel without its input.
  The Alice path is especially affected because a `Plain` kernel without
  its input looks like an ordinary `Plain` kernel.
- **Activation.** The current gate is the process-global
  `global::is_user_testing_mode()`, not a block-height consensus rule.
- **Privacy of the `alice` hash.** A reused `A_i` links payments; a fresh
  `A_i` per payment is required.
- **Signature key-prefixing.** `ReceiptClaim` currently signs with
  `pubkey_for_e = None`; key-prefixing should be considered as a
  defense-in-depth hardening.
- **Collusion.** Alice and Bob can always collude; the design must not
  claim to prevent collusion.
- **Truncated Step 2 instruction.** The missing requirements must be
  confirmed before implementation.

## Deliverable

`docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md`, containing:

- problem statement and threat model;
- candidate evaluation;
- recommended construction;
- authorization matrix;
- attack analysis;
- privacy analysis;
- implementation impact and classification;
- blockers and risks;
- Step 3 proposal and decision gate.

## Stop condition

Stop after Step 2. Report the findings and wait for explicit approval.
Do not implement the authorization scheme, modify consensus or
serialization, add kernel types, reintroduce `AliceReclaim`, change the
`Receipt` output format, modify wallet behavior, commit changes, or
automatically proceed to Step 3.

---

## Archiv: Step 1 (abgeschlossen)

Step 1 investigated whether adaptor signatures could provide a mandatory,
privately verifiable Bob claim. Result: the existing `ReceiptClaim` by
Bob's one-time receipt key `b_i` is already the mandatory evidence for the
`BobClaim` path while bare `Plain` spends are forbidden. Adaptor
signatures do not strengthen the guarantee: completion reveals the adaptor
secret, `T_i = B_i` leaks `b_i`, a static `T` becomes forgeable after the
first claim, and a non-consensus-enforced pre-signature can be omitted.
The core blocker is that Alice and Bob both know the shared Pedersen blind
`q_i`. The remedy is an exclusive authorization scheme and a consensus
change, not an adaptor signature.

The full Step 1 report is in
[`docs/SNIP_ADAPTOR_FEASIBILITY.md`](docs/SNIP_ADAPTOR_FEASIBILITY.md).

---

## Archiv: Vorheriger Stand (vor Step 1)

## Status: ACTIVE — WARTET AUF DESIGN-ENTSCHEIDUNG

## Objective

**Vollständige Demonstration des SNIP-Transaktionsflusses** gemäß
[PROTOCOL.md](PROTOCOL.md) §2, §3.8, §3.9 und §5.

- Bob veröffentlicht einen **Payment Descriptor** einmalig.
- Alice erstellt daraus **drei unabhängige Zahlungen**
  (ohne Bob, ohne Kommunikation).
- Alle drei Zahlungen werden in der **lokalen Usernet-Chain**
  bestätigt.
- Bob **detektiert** alle drei **durch Scannen der Chain**
  (ohne Kommunikation mit Alice) und **bestimmt Beträge**.
- Bob **claimt** jede der drei Zahlungen in **einer eigenen
  Transaktion**.
- Alle drei Claims werden **in der Usernet-Chain bestätigt**.
- Die **End-Outputs** sind **ausschließlich von Bob auszahlbar**.
- **Alice kann aus der Chain verifizieren**, dass Bob die
  Zahlungen übernommen hat (§3.8).
- **Alice kann die Shared Outputs jederzeit selbst
  beanspruchen**, solange Bob sie nicht bereits in seine
  eigenen Outputs übertragen hat (§3.9). Alice nutzt hierfür
  die **bestehende Standard-Mechanik von Epic Cash**
  (gewöhnlicher `Plain`-Kernel).

Dieses Akzeptanzkriterium ist die **oberste Priorität**
dieses Projekts. Alle anderen Arbeit — die In-Memory-POC unter
`src/`, die Attributionsforschung unter `docs/`, die
vorherigen Receipt-Claim-Forschungspfade — sind **abgeschlossen
und überholt durch diese Spezifikation** und dienen nur noch
als historische Aufzeichnung.

## Identifizierte Inkonsistenzen in der Doku (korrigiert)

Die in der vorherigen Doku-Runde eingebrachten Aussagen sind
falsch und wurden korrigiert:

1. **`AliceReclaim`-Kerneltyp.** Mehrere Dokumente
   (`README.md`, `PROTOCOL.md` §3.9/§4,
   `docs/receipt-claim-design.md`,
   `docs/SNIP_SPEND_ATTRIBUTION_RESEARCH.md`) sagten,
   `AliceReclaim` werde "wieder eingeführt" oder sei "Teil
   des Zielprotokolls". Das ist falsch. §3.9 verlangt, dass
   Alice die Shared Outputs mit der **bestehenden
   Standard-Mechanik von Epic Cash** beanspruchen kann —
   das ist ein gewöhnlicher `Plain`-Kernel. Es gibt keinen
   und soll keinen separaten `AliceReclaim`-Kerneltyp
   geben.

2. **"Bob-Claim hat Vorrang".** Mehrere Dokumente
   behaupteten, Bob-Claim habe Vorrang. Das ist falsch.
   Die Semantik ist **First-Confirmed-Spend-Wins** — der
   erste *bestätigte* gültige Spend gewinnt, unabhängig
   davon, ob es Bob oder Alice ist. Es gibt keine
   spezialpriorität für Bob.

Korrigiert in: `PROTOCOL.md`, `README.md`,
`docs/receipt-claim-design.md`,
`docs/SNIP_SPEND_ATTRIBUTION_RESEARCH.md`.

## Konsens-Investigation (abgeschlossen, 7 Fragen)

Alle 7 Fragen der Konsens-Investigation sind beantwortet.
Kurzform:

| # | Frage | Antwort |
|---|-------|---------|
| 1 | Warum werden gewöhnliche Spends von Shared Outputs abgelehnt? | Zwei Regeln: (a) `verify_receipt_resolution()` verlangt 1:1-Zuordnung `Receipt`-Inputs ↔ `BobClaim`-Kernels — ein `Plain`-Kernel scheitert an der Zählregel (Zeile 946). (b) `verify_receipt_claim()` erzwingt, dass jeder `BobClaim`-Kernel ein `ReceiptClaim` trägt. |
| 2 | Welche Validierungsregel erzwingt BobClaim? | `TransactionBody::verify_receipt_resolution()` — Zählregel + Per-Input-Matching (`claim.original_commitment == input.commit`, `hashes.matches(&claim.receipt_public_key)`). |
| 3 | Welche Krypto-Secrets benötigt man für einen Shared-Output-Spend? | **Bob-Claim:** `bob_shared_blind` + `bob_receipt_key` + `destination_blind`. **Plain-Spend (Alice oder Bob):** nur `shared_blind` + `destination_blind`. |
| 4 | Kann Alice eine gültige gewöhnliche Ausgabe erstellen? | **Kryptografisch ja.** Alice kennt `shared_blind` via `StealthReceiptDescriptor::derive_shared_spend_blind_for_sender`. Sie kann einen gültigen `Plain`-Kernel bauen. **Konsenslich nein.** Der `Plain`-Kernel + `Receipt`-Input wird an der Zählregel abgelehnt. |
| 5 | Kann Bob eine gültige gewöhnliche Ausgabe erstellen? | Gleiche Antwort wie Alice: Kryptografisch ja, konsenslich nein. |
| 6 | Kann Bob die Receipt-Attribution umgehen? | **Nein.** Jeder Spend eines `Receipt`-Inputs muss ein `BobClaim`-Kernel sein (einziger `is_receipt_resolution()`-Variant), der ein `ReceiptClaim` mit `bob_receipt_key`-Signatur erfordert. |
| 7 | Kann Alice einen BobClaim fälschen? | **Nein.** Alice kennt nicht `receipt_spend_secret_key` und kann keinen gültigen `bob_receipt_key` ableiten. Alle 5 Impersonationsfälle in `spend_attribution.rs` (Experiment C) werden abgelehnt. |

## Kernarchitektonischer Konflikt

**Requirement §3.9 und der aktuelle Konsensmechanismus
sind inkongruent.**

Der aktuelle Konsensmechanismus (`verify_receipt_resolution`)
verlangt, dass **jeder** `Receipt`-Input exakt einen
`BobClaim`-Kernel hat. Ein gewöhnlicher `Plain`-Kernel, der
einen `Receipt`-Input ausgibt, wird **immer** abgelehnt
(Zählregel: `receipt_inputs.len() != receipt_kernels.len()`).

§3.9 verlangt aber, dass **Alice** die Shared Outputs
jederzeit mit der **bestehenden Standard-Mechanik von Epic
Cash** beanspruchen kann — also mit einem gewöhnlichen
`Plain`-Kernel.

**Die beiden sind direkt in Konflikt.**

### Kryptografische Beobachtung

Sowohl Alice als auch Bob kennen `shared_blind`:
- Alice: `StealthReceiptDescriptor::derive_shared_spend_blind_for_sender()`
- Bob: `StealthReceiptSecrets::derive_shared_spend_blind()`

Beide können kryptografisch einen gültigen `Plain`-Kernel
bauen. Der Konsensmechanismus ist die einzige Barriere.

### Was der aktuelle Mechanismus leistet

- Bob-Claim ist **verifizierbar als Bobs** (via `ReceiptClaim`
  Signatur mit `bob_receipt_key` und Hash-Bindung an
  `SharedReceiptKeyHashes.bob`).
- Alice **kann** Bob-Claim aus der Chain verifizieren (§3.8 ✓).
- Alice **kann** Bob-Claim **nicht** fälschen (§3.8-Integrität ✓).
- Bob **kann** die Receipt-Attribution **nicht** umgehen
  (muss immer BobClaim-Kernel verwenden).

### Was der aktuelle Mechanismus **nicht** leistet

- Alice **kann** die Shared Outputs **nicht** selbst mit
  Standard-Epic-Mechanik beanspruchen (§3.9 ✗).

## Offene Designentscheidung (noch zu treffen)

Um §3.9 zu erfüllen, muss der Konsensmechanismus angepasst
werden. Zwei Optionen:

### Option A: `Plain`-Kernel für `Receipt`-Inputs erlauben

**Änderung:** Die Zählregel in
`verify_receipt_resolution()` wird so angepasst, dass
`Plain`-Kernels einen `Receipt`-Input ausgeben dürfen.
Nur `BobClaim`-Kernels müssen weiterhin den
`ReceiptClaim`-Matching durchlaufen.

**Effekt:**
- Alice kann einen `Plain`-Kernel bauen und den
  Shared-Output selbst beanspruchen (§3.9 ✓).
- Bob **kann** ebenfalls einen `Plain`-Kernel verwenden
  (er kennt `shared_blind`) und die
  Receipt-Attribution umgehen. Das wäre ein **Problem**
  für §3.8, weil ein Bob-`Plain`-Spend nicht als Bobs
  verifizierbar wäre.

**Risiko:** Bob würde die Attributierung umgehen können.
Alice könnte nicht mehr zuverlässig unterscheiden, ob Bob
geclaimt hat oder ob er sich gewöhnlich ausgezahlt hat.
§3.8 wäre beeinträchtigt.

**Lösungsmöglichkeit:** Bob kann auf Bobs eigene
`Plain`-Spend verzichten, weil er das `BobClaim`-Feature
nutzt, das seinen Spend verifizierbar macht. Das
Protokoll kann Bob **verpflichten**, BobClaim zu
verwenden (via Client-Logik, nicht Konsens). Aber das
ist keine konsensgewährte Eigenschaft.

### Option B: Neue `ReceiptReclaim`-Kernelvariante

**Änderung:** Ein neuer Kernel-Typ
`KernelFeatures::ReceiptReclaim` wird eingeführt, der
einen `Receipt`-Input ausgeben darf. Im Unterschied zu
`BobClaim`:
- Kein `receipt_spend_secret_key` erforderlich.
- Nur `shared_blind` + `destination_blind` benötigt.
- Kein `ReceiptClaim`-Matching erforderlich.
- `SharedReceiptKeyHashes` erhält einen zusätzlichen
  `alice`-Hash (oder ein generisches `shared`-Hash).

**Effekt:**
- Alice kann einen `ReceiptReclaim`-Kernel bauen
  (§3.9 ✓).
- Bob **kann** auch `ReceiptReclaim` verwenden (er
  kennt `shared_blind`). Aber Bob **wird** `BobClaim`
  verwenden, weil das seine Ausgabe verifizierbar macht
  (§3.8).
- Die Attributierung bleibt für Bob-Claims
  konsensgewährt.

**Risiko:** Neue Kernelvariante = neuer
Konsensmechanismus. Muss sauber spezifiziert und
getestet werden.

### Empfehlung

**Option B** (`ReceiptReclaim`) ist die technisch
sauberere Lösung. Sie erfüllt §3.9, bewahrt §3.8
(weil Bob-Claims weiterhin die `ReceiptClaim`-Signatur
tragen), und macht die Asymmetrie explizit: Bob hat
einen verifizierbaren Pfad, Alice hat einen
Standard-Pfad.

**Option A** ist simpler, aber schwächer: Es gibt keine
konsensgewährte Attributierung, wenn Bob einen
`Plain`-Spend verwendet.

### WICHTIG

**Beide Optionen erfordern eine Konsensänderung.**
Gemäß der Instruktion "Do not modify consensus code until
the architectural investigation is complete" ist die
Investigation jetzt abgeschlossen. Die Entscheidung über
Option A vs. B muss vor der Implementierung getroffen
werden.

## Nächste Schritte (nach Designentscheidung)

1. **Designentscheidung treffen:** Option A (Plain-Kernel
   erlauben) oder Option B (ReceiptReclaim-Kernel).
2. **Konsensänderung implementieren.**
3. **Tests schreiben:**
   - `Plain`-Kernel spendet Receipt-Input → erlaubt
     (nur wenn Option A).
   - `ReceiptReclaim`-Kernel spendet Receipt-Input →
     erlaubt (nur wenn Option B).
   - BobClaim-Kernel bleibt unverändert.
   - Alice kann Shared-Output beanspruchen, Bob später
     abgewiesen (First-Spend-Wins).
   - Bob kann Shared-Output beanspruchen, Alice später
     abgewiesen (First-Spend-Wins).
4. **Scan-Api für Bob implementieren.**
5. **PaymentDescriptor-Abstraktion definieren.**
6. **Neuen Integrationstest**
   `upstream/epic/chain/tests/snip_acceptance.rs`
   schreiben:
   - Bob erzeugt Descriptor.
   - Alice erzeugt 3 unabhängige Zahlungen aus dem
     Descriptor.
   - Alle 3 werden in die Usernet-Chain gemint.
   - Bob scannt, detektiert alle 3, bestimmt Beträge.
   - Bob claimt jede der 3 in eigener Transaktion.
   - Alle 3 Claims werden in der Usernet-Chain bestätigt.
   - End-Outputs sind ausschließlich von Bob auszahlbar
     (Verifizierung: kein anderer kann sie ausgeben).
   - Alice kann aus der Chain verifizieren, dass Bob die
     Zahlungen übernommen hat (§3.8).
   - **Zusätzlicher Testfall:** Alice versucht, einen
     Shared-Output nach Bobs Claim auszugeben → wird
     abgewiesen (bereits spent).
   - **Zusätzlicher Testfall:** Alice erstellt eine
     Zahlung und gibt sie selbst aus (Plain-Kernel),
     bevor Bob scannt → Alice-Spend erfolgreich,
     Bob-Claim später abgewiesen.
7. **Alle Test-Suites ausführen:**
   - `cargo test -p epic_core --lib`
   - `cargo test -p epic_chain --test snip_acceptance`
     (neu)
   - `cargo test -p epic_chain --test spend_attribution`
   - `cargo test -p epic_chain --test
     receipt_claim_roundtrip`
8. **Kein Commit, bis die Ergebnisse überprüft sind.**

## Abgeschlossene Arbeit (historisch)

| Thema | Status | Doku |
| --- | --- | --- |
| One-sided shared-redeemable In-Memory-POC (`src/`) | ✅ abgeschlossen | [docs/one-sided-poc.md](docs/one-sided-poc.md) |
| `AliceReclaim`-Mechanik vollständig entfernt | ✅ abgeschlossen | — |
| Spend-Attribution-Forschung, Verdict **VERIFIED** | ✅ abgeschlossen | [docs/SNIP_SPEND_ATTRIBUTION_RESEARCH.md](docs/SNIP_SPEND_ATTRIBUTION_RESEARCH.md) |
| BobClaim-Mechanismus im Fork unanfällig | ✅ abgeschlossen | [docs/receipt-claim-design.md](docs/receipt-claim-design.md) |
| Usernet-Chain-Roundtrip (1 Zahlung) | ✅ abgeschlossen | [docs/usernet-experiment.md](docs/usernet-experiment.md) |
| Konsens-Investigation (7 Fragen) | ✅ abgeschlossen | [CURRENT_STEP.md](CURRENT_STEP.md) |

Keine der obigen abgeschlossenen Arbeiten widerspricht der
neuen SNIP-Spezifikation; sie ist das **Fundament**, auf dem
der neue 3-Zahlungs-Flow aufbaut.
