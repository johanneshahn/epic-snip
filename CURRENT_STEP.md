# CURRENT_STEP

## Status: STEP 3B ERLEDIGT — ABGESCHLOSSEN

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
