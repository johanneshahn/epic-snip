# SNIP Adaptor-Signature Feasibility

**Status: Step 1 feasibility report — investigation only.**
**Scope: local Usernet research fork. Not a consensus specification.
Not for Mainnet, Floonet, or real funds.**

This report investigates whether Epic SNIP can implement a mandatory,
privately verifiable Bob claim using adaptor-signature techniques and the
existing `Receipt` output type. It does **not** implement the protocol and
does **not** modify consensus, chain, or `libtx` code.

---

## Verdict (short form)

| Question | Answer |
|---|---|
| Does the existing `Receipt`/`BobClaim` design already provide mandatory Bob evidence? | **Yes, for the `BobClaim` path, while ordinary `Plain` spends of `Receipt` inputs are forbidden.** The evidence is the `ReceiptClaim` Schnorr signature by Bob's one-time receipt key `b_i`. |
| Do adaptor signatures make that evidence stronger or more private? | **No.** A completed adaptor signature reveals its adaptor secret by construction. If the secret is `b_i`, the claim becomes publicly front-runnable. If it is an independent static secret, the first claim reveals it and future evidence is forgeable. If the pre-signature is not consensus-enforced, Bob can omit it. |
| Can a reusable descriptor supply a per-payment adaptor pre-signature? | **No.** A Schnorr adaptor pre-signature is message- and nonce-specific. A pre-signature produced at claim time is not a pre-commitment and adds no security. |
| Can adaptor signatures separate Alice's and Bob's spending paths? | **No.** Alice and Bob both know the Shared Output blind `q`. Any rule that lets Alice spend with `q` also lets Bob spend with `q`. Separating the paths requires an exclusive secret per party plus a consensus-visible authorization, or a script/MAST-like OR commitment. |
| Can the Pedersen commitment and Bulletproof validation stay unchanged? | **Yes.** The Shared Output is a standard Pedersen commitment with a standard Bulletproof. |
| What is the core blocker? | **`UNRESOLVED_CRYPTOGRAPHIC_PROBLEM`** under the current key model: one Pedersen commitment with one shared blinding secret cannot express an OR of two independent spending keys, and adaptor signatures reveal their secret on completion. The remedy is a **`REQUIRES_CONSENSUS_CHANGE`** authorization scheme, not an adaptor signature. |

The rest of this document supports these conclusions with exact source
references and cryptographic reasoning.

---

## A. Current implementation

### A.1 Output type and serialized representation

The experimental output feature is `OutputFeatures::Receipt = 2`:

- `upstream/epic/core/src/core/transaction.rs:1510-1522` —
  `OutputFeatures` enum; `Receipt = 2` is marked
  "Experimental Usernet-only output carrying two receipt-key hashes".
- `upstream/epic/core/src/core/transaction.rs:1543-1563` —
  `Output` gains two optional fields:
  - `receipt_key_hashes: Option<SharedReceiptKeyHashes>`
  - `receipt_stealth_hint: Option<ReceiptStealthHint>`
- `upstream/epic/core/src/core/transaction.rs:1578-1597` —
  `Output::write` serializes the two receipt fields **only** when
  `features.is_receipt()`. The range proof is omitted in
  `SerializationMode::Hash` and written otherwise.
- `upstream/epic/core/src/core/transaction.rs:1601-1640` —
  `Output::read` reads the two fields only for `Receipt`.
- `upstream/epic/core/src/core/transaction.rs:1400-1472` —
  `Input` carries the same two optional fields and serializes them only
  for `Receipt`.
- `upstream/epic/core/src/core/transaction.rs:1696-1706` and
  `:1767-1780` — `OutputIdentifier` carries the same two fields and
  serializes them only for `Receipt`.
- `upstream/epic/core/src/receipt_claim.rs:28-98` —
  `ReceiptStealthHint` is a 33-byte compressed secp256k1 public key
  (`ephemeral_public_key`), serialized as fixed 33 bytes and validated
  on read.
- `upstream/epic/core/src/receipt_claim.rs:521-527` —
  `SharedReceiptKeyHashes { bob: [u8; 32] }`, serialized as 32 bytes.

The raw Bob receipt public key `B_i` is **not** stored in the output.
Only its domain-separated hash `H(B_i)` is stored:

- `upstream/epic/core/src/receipt_claim.rs:554-570` —
  `SharedReceiptKeyHashes::from_public_key` and `matches`.
- `upstream/epic/core/src/receipt_claim.rs:587-593` —
  `receipt_key_hash(key) = SHA256(DOMAIN || "receipt-key|" ||
  compressed(key))`, with
  `DOMAIN = b"EPIC-SNIP-USERNET-v1|receipt-claim|"` (line 22).

### A.2 Pedersen commitment and range proof

The Shared Output is a normal Pedersen commitment with a **shared**
blinding factor:

- `upstream/epic/core/src/receipt_claim.rs:282-308` —
  `build_shared_receipt_output` calls
  `secp.commit(value, shared_blind)` and
  `secp.bullet_proof(value, shared_blind, rewind_nonce, ...)`.
- `upstream/epic/core/src/receipt_claim.rs:502-519` —
  `shared_rewind_nonce` derives the Bulletproof rewind nonce from the
  shared blind and the commitment. This is a prototype convenience, not
  a consensus rule.
- `upstream/epic/core/src/receipt_claim.rs:313-326` —
  `rewind_shared_receipt_output` rewinds the standard Bulletproof with
  that nonce.

There is **no custom proof system**. Validation uses the ordinary
`Output::batch_verify_proofs` path in
`upstream/epic/core/src/core/transaction.rs:994-1006`.

### A.3 Shared blind and Bob's one-time receipt key

Alice and Bob derive the same Mimblewimble blind `q_i` by ECDH:

- `upstream/epic/core/src/receipt_claim.rs:253-274` —
  `shared_spend_blind(secp, secret, peer, ephemeral, scan_public_key)`
  computes `SharedSecret::new(secp, peer, secret)` and hashes
  `DOMAIN || "shared-spend-blind|" || shared || ephemeral || scan_pubkey
  || counter`.
- `upstream/epic/core/src/receipt_claim.rs:209-223` —
  Alice's `StealthReceiptDescriptor::derive_shared_spend_blind_for_sender`.
- `upstream/epic/core/src/receipt_claim.rs:160-174` —
  Bob's `StealthReceiptSecrets::derive_shared_spend_blind`.

Bob's one-time receipt key is derived with a **different** domain:

- `upstream/epic/core/src/receipt_claim.rs:226-247` —
  `receipt_tweak` hashes
  `DOMAIN || "stealth-receipt-tweak|" || shared || ephemeral ||
  receipt_spend_pubkey || counter`.
- `upstream/epic/core/src/receipt_claim.rs:180-200` —
  Alice's `StealthReceiptDescriptor::derive_for_sender` computes
  `B_i = P + h_i G` (using `PublicKey::add_exp_assign`).
- `upstream/epic/core/src/receipt_claim.rs:132-151` —
  Bob's `StealthReceiptSecrets::derive_receipt_key` computes
  `b_i = p + h_i`.

Alice can compute `B_i` but **not** `b_i`. Bob can compute both. Both can
compute `q_i`. This asymmetry is the foundation of the current design and
the source of the Alice-path problem analyzed in §C and §D.

### A.4 BobClaim kernel

- `upstream/epic/core/src/core/transaction.rs:41-58` —
  `KernelFeatures::BobClaim { fee }`, discriminant `3`
  (`BOB_CLAIM_U8` at line 64).
- `upstream/epic/core/src/core/transaction.rs:100-113` —
  `kernel_sig_msg` for `BobClaim` is `hash(feature_byte, fee)`. It does
  **not** include the kernel excess; the excess is bound through the
  signature's public key instead.
- `upstream/epic/core/src/core/transaction.rs:416-426` —
  `is_receipt_resolution()` and `receipt_path()` return
  `Some(ReceiptPath::BobClaim)` only for `BobClaim`.
- `upstream/epic/core/src/core/transaction.rs:312-340` —
  `TxKernel` has `receipt_claim: Option<ReceiptClaim>`.
- `upstream/epic/core/src/core/transaction.rs:342-374` —
  the `receipt_claim` is serialized/deserialized only for
  `is_receipt_resolution()` kernels.

### A.5 ReceiptClaim authorization data

- `upstream/epic/core/src/receipt_claim.rs:595-622` —
  `ReceiptPath::BobClaim` is the only path. The tag is `0`.
- `upstream/epic/core/src/receipt_claim.rs:627-645` —
  `ReceiptClaim` fields:
  - `path: ReceiptPath`
  - `original_commitment: Commitment`
  - `kernel_excess: Commitment`
  - `fee: u64`
  - `receipt_public_key: PublicKey` (this is `B_i`)
  - `receipt_signature: Signature` (Schnorr by `b_i`)
- `upstream/epic/core/src/receipt_claim.rs:650-663` —
  the signed message is
  `SHA256(DOMAIN || path_tag || original_commitment ||
  kernel_excess || fee_le)`.
- `upstream/epic/core/src/receipt_claim.rs:665-696` —
  `sign` uses `secp::aggsig::sign_single(..., None, None, None, None,
  None)`, i.e. a random nonce and **no** `pubkey_for_e`.
- `upstream/epic/core/src/receipt_claim.rs:698-721` —
  `verify` uses `secp::aggsig::verify_single(..., None,
  &receipt_public_key, None, None, false)`.

Cryptographic observation: because `sign_single` is called with
`pubkey_for_e = None`, the `ReceiptClaim` challenge is
`e = H(R.x || message)` and **not** `H(R.x || B_i || message)`. The
kernel signature, by contrast, uses
`pubkey_for_e = Some(&excess_public_key)`
(`upstream/epic/core/src/receipt_claim.rs:385-394` and `:478-488`), so
its challenge is `H(R.x || X || message)`. The `ReceiptClaim` signature
is still bound to `B_i` through the verification equation
`sG = R + e B_i`, but the key is not key-prefixed into the challenge.
This is a consistency/defense-in-depth issue for Step 2, not a break of
the current construction.

### A.6 Consensus validation

`TransactionBody::verify_receipt_claim`:

- `upstream/epic/core/src/core/transaction.rs:488-504` — for a kernel
  with a receipt path:
  - `claim.path == path`
  - `claim.kernel_excess == self.excess`
  - `claim.fee == self.features.fee()`
  - `claim.verify()`
  Otherwise `Error::InvalidReceiptClaim`.

`TransactionBody::verify_receipt_resolution`:

- `upstream/epic/core/src/core/transaction.rs:897-976`:
  1. Every input/output must have consistent metadata:
     `Receipt` ⇔ both `receipt_key_hashes` and `receipt_stealth_hint`
     are `Some`; non-`Receipt` ⇔ both are `None`.
  2. The stealth hint must parse as a valid compressed public key.
  3. `receipt_inputs.len() == receipt_kernels.len()`.
  4. Each `Receipt` input must match **exactly one** `BobClaim` kernel
     with `claim.original_commitment == input.commit`.
  5. `input.receipt_key_hashes.matches(&claim.receipt_public_key)`.

`TransactionBody::verify_receipt_feature_activation`:

- `upstream/epic/core/src/core/transaction.rs:887-894` — any receipt
  feature is rejected unless `global::is_user_testing_mode()` is true.
  This is a process-global gate, not a block-height activation rule.

Validation call sites:

- `upstream/epic/core/src/core/transaction.rs:982-989` —
  `validate_read` calls `verify_weight`, `verify_sorted`,
  `verify_cut_through`, `verify_receipt_feature_activation`, and
  `verify_receipt_resolution`.
- `upstream/epic/core/src/core/transaction.rs:994-1012` —
  `validate` additionally batch-verifies range proofs and kernel
  signatures (`TxKernel::batch_sig_verify`).
- `upstream/epic/core/src/core/transaction.rs:507-532` —
  `batch_sig_verify` verifies every kernel signature, then calls
  `verify_receipt_claim` for every kernel.

### A.7 Why ordinary `Plain` spends cannot consume `Receipt` outputs

There are two layered rules:

1. **Count rule.** `verify_receipt_resolution` requires
   `receipt_inputs.len() == receipt_kernels.len()`
   (`transaction.rs:946`). A `Plain` kernel is not
   `is_receipt_resolution()`, so a transaction with one `Receipt` input
   and one `Plain` kernel has `1 != 0` and is rejected.
2. **Evidence rule.** Every `BobClaim` kernel must carry a valid
   `ReceiptClaim` (`transaction.rs:488-504`). There is no `Plain` path
   that carries a `ReceiptClaim`.

The chain-level research test
`upstream/epic/chain/tests/spend_attribution.rs` confirms this: a
`Plain` kernel spending a `Receipt` input is rejected with
`InvalidReceiptClaim` (see the comments and assertions around
`spend_attribution.rs:303-369`). The `spend_attribution.rs` test also
confirms that Alice cannot forge a `BobClaim` because she cannot derive
`b_i` (`:376-486`).

### A.8 Existing helpers and tests

- `upstream/epic/core/src/receipt_claim.rs:334-425` —
  `build_bob_claim_transaction`:
  - `destination_value = receipt_value - fee`
  - `destination_commit = commit(destination_value, destination_blind)`
  - `kernel_secret = destination_blind - shared_blind`
  - `excess = commit(0, kernel_secret)`
  - kernel signature by `kernel_secret` over `hash(3, fee)`
  - `ReceiptClaim::sign(BobClaim, receipt_output.commit, excess, fee,
    bob_receipt_key)`
  - input carries the original receipt metadata.
- `upstream/epic/core/src/receipt_claim.rs:434-500` —
  `build_receipt_funding_transaction`: Alice's funding transaction with
  a `Plain` kernel and a `Receipt` output.
- `upstream/epic/core/src/libtx/build.rs:162-...` —
  `receipt_output` helper.
- `upstream/epic/core/src/libtx/aggsig.rs` — AgSig wrappers.
- `upstream/epic/core/src/receipt_claim.rs:747-894` — unit tests for
  signature binding, serialization, ECDH derivations, shared blind
  equality, and a full `BobClaim` transaction validation.
- `upstream/epic/chain/tests/receipt_claim_roundtrip.rs` — one-payment
  Usernet chain roundtrip.
- `upstream/epic/chain/tests/spend_attribution.rs` — attribution
  experiments A/B/C, including the `Plain`-spend rejection and Alice's
  forged-`BobClaim` rejection.
- `upstream/epic/core/examples/receipt_stealth_demo.rs` — end-to-end
  derivation demo.
- Top-level POC: `src/crypto.rs`, `src/one_sided.rs` — no adaptor
  signatures. `grep` for `adaptor|Adaptor|pre_sig|presign|pre-sign`
  finds only `servers/src/common/adapters.rs` (network adaptor, unrelated).

### A.9 Reusable components

The following components are reusable as-is:

- The ECDH + domain-separated hash derivation of `q_i` and `b_i`.
- The `B_i = P + h_i G` construction and the `H(B_i)` binding.
- The standard Pedersen commitment and Bulletproof.
- The Schnorr `ReceiptClaim` signature and its binding to
  `original_commitment`, `kernel_excess`, and `fee`.
- The `BobClaim` kernel feature and `verify_receipt_resolution` matching.
- The `ReceiptStealthHint` scan mechanism.
- The AgSig key arithmetic and signature verification.

The following components are **not** reusable for the Alice path:

- The shared blind `q_i` cannot be made Alice-exclusive without changing
  the key model.
- There is no script/OR mechanism on the kernel excess.
- There is no per-path authorization field for Alice.

---

## B. Cryptographic construction

### B.1 Available primitives

The Epic stack uses `grin_secp256k1zkp` 0.8.1
(`upstream/epic/util/Cargo.toml:35-38`,
`Cargo.lock:1799-1803`) and re-exports it as `util::secp`
(`upstream/epic/util/src/lib.rs:35`).

The AgSig signature is 64 bytes: `R.x || s` (`ffi::Signature` is
`[u8; 64]`, `constants::AGG_SIGNATURE_SIZE = 64`). The challenge is

```
e = SHA256(R.x || X_compressed || msg32)        (when pubkey_for_e is provided)
e = SHA256(R.x || msg32)                        (when pubkey_for_e is NULL)
```

and the signature equation is

```
s = k + e*x  (mod n),        sG = R + eX.
```

The signer normalizes the nonce to the "quadratic y" convention; the
verifier reconstructs `R` from `R.x` with the same convention. This is
implemented in the vendored C source
`depend/secp256k1-zkp/src/modules/aggsig/main_impl.h`:
`secp256k1_compute_sighash_single` and
`secp256k1_aggsig_verify_single`.

Available Rust-level operations (vendored crate):

| Primitive | Location | Status |
|---|---|---|
| `aggsig::sign_single` with explicit `secnonce`, `extra`, `pubnonce`, `pubkey_for_e`, `final_nonce_sum` | `src/aggsig.rs:88-145` | Exists |
| `aggsig::verify_single` with explicit `pubnonce`, `pubkey_total_for_e`, `extra_pubkey`, `is_partial` | `src/aggsig.rs:147-196` | Exists |
| `aggsig::verify_batch` | `src/aggsig.rs:198-237` | Exists |
| `aggsig::add_signatures_single` | `src/aggsig.rs:239-265` | Exists |
| `aggsig::subtract_partial_signature` | `src/aggsig.rs:267-301` | Exists |
| `AggSigContext::partial_sign`, `combine_signatures`, `verify` | `src/aggsig.rs:303-401` | Exists |
| `SecretKey::{add_assign, mul_assign, inv_assign, neg_assign}` | `src/key.rs:101-150` | Exists |
| `PublicKey::{from_combination, add_exp_assign, mul_assign}` | `src/key.rs:160-286` | Exists |
| `Signature::from_raw_data`, `AsRef<[u8]>` | `src/lib.rs:73-120, 146` | Exists |
| High-level adaptor API (pre-sign / verify / complete / extract) | — | **Does not exist** |

There is **no** `adaptor` module in the vendored library. However, the
nonce-controlled `sign_single`/`verify_single` and the key arithmetic are
sufficient to implement Schnorr adaptor signatures **in Rust**, without
changing the C library. A high-level API would be a local Rust module
(e.g. `core/src/libtx/adaptor.rs`), not a change to the vendored crate.
This is `FEASIBLE_WITH_LOCAL_CHANGES` at the cryptographic-library level.

### B.2 Candidate construction 1: per-payment adaptor point `T_i = B_i`

**Setup.**

- Bob publishes the reusable descriptor `(S = sG, P = pG)`.
- For payment `i`, Alice picks an ephemeral secret `r_i`, computes
  `R_i = r_i G`, the hint `R_i`, the receipt key
  `B_i = P + h_i G`, and the shared blind `q_i`.
- Alice creates
  `C_i = v_i G + q_i H` and stores `(H(B_i), R_i)` in the output.
- Bob scans, computes `b_i = p + h_i` and `q_i`, and verifies
  `H(B_i) == output.receipt_key_hashes.bob`.

**Claim.**

- Bob chooses destination blind `d_i`, destination value
  `v_i - fee`, destination commitment
  `C_dest = (v_i - fee)G + d_i H`.
- Kernel excess `E = C_dest - C_i = (d_i - q_i)G`.
- Kernel secret `x = d_i - q_i`.
- Kernel message `m = hash(3, fee)`.
- Standard kernel signature: nonce `k`, `R = kG`,
  `e = H(R.x || X || m)`, `s = k + e*x`, where `X = xG = E.to_pubkey()`.
- `ReceiptClaim` by `b_i` over
  `(BobClaim, C_i, E, fee)`, as today.

**Adaptor variant.**

- Adaptor point `T_i = B_i`, adaptor secret `t_i = b_i`.
- Choose `k`, `R = kG`, `R* = R + T_i`.
- `e = H(R*.x || X || m)`.
- Pre-signature `s' = k + e*x`.
- Completed signature `s = s' + t_i`.
- The kernel signature is `(R*.x, s)`, which is a valid Schnorr
  signature under `X` because
  `sG = (k + e*x + t_i)G = R + eX + T_i = R* + eX`.
- The pre-signature check is
  `s'G = R + eX = R* + eX - T_i = sG - T_i`, equivalently
  `(s - s')G = T_i`.

**What Alice would inspect.** The kernel signature is 64 bytes:
`R*.x` (bytes 0..32) and `s` (bytes 32..64). The pre-signature `s'`
would be an extra 32-byte scalar. Alice would:

1. Read `s` from the kernel signature.
2. Read `s'` from the new kernel field.
3. Compute `t_i = s - s'` (mod n).
4. Check `t_i G == B_i` (where `B_i` is in the `ReceiptClaim`).
5. Check `H(B_i) == input.receipt_key_hashes.bob`.
6. Check the `ReceiptClaim` signature as today.

**Why it fails.**

1. **The evidence is public, not private.** `s` and `s'` are both
   on-chain, so anyone can compute `t_i = b_i`. The "private"
   verification becomes public disclosure. An observer who sees the
   claim in the mempool can compute `b_i` and build a competing claim
   with a higher fee. Since the protocol is first-confirmed-spend-wins,
   this is a real front-running vector.
2. **The equation is trivially satisfiable by Bob.** For any valid
   signature `s`, Bob can set `s' = s - b_i`. The consensus check
   `(s - s')G == B_i` then passes. It does not force Bob to use a
   particular nonce, to have prepared anything in advance, or to have
   used the adaptor nonce `R*`. It only forces him to reveal `b_i`.
3. **It is redundant with the existing `ReceiptClaim`.** The existing
   signature already proves knowledge of `b_i`, bound to the same
   `original_commitment`, `kernel_excess`, and `fee`, **without**
   revealing `b_i`. The adaptor variant is strictly worse.
4. **It does not solve the Alice path.** Alice still cannot produce the
   adaptor evidence (she lacks `b_i`), and if a `Plain` path is allowed
   she does not need it. Bob can still use the `Plain` path with `q_i`.

**Implementation note.** The vendored `sign_single` stores the
x-coordinate of the secret nonce it is given; it does not accept a
separate "adaptor nonce" that differs from the secret nonce. In the
`T_i = B_i` construction Bob knows `t_i`, so he can set the secret
nonce to `k + t_i` (producing `R*.x`) or simply compute
`s' = s - t_i` after signing. This is another way to see that the
adaptor equation adds no commitment: the "pre-signature" can always be
derived from a completed signature by subtracting `t_i`. A PoC must not
assume that passing `pubnonce_for_e = Some(R*)` alone makes the signer
produce an adaptor nonce; the nonce normalization and stored `R.x`
semantics must be tested explicitly (§H.3, test 6).

### B.3 Candidate construction 2: static adaptor point `T = tG`

**Setup.** Bob publishes a long-term adaptor point `T = tG` in the
descriptor. Every claim uses `R* = R + T`, `s' = k + e*x`,
`s = s' + t`, and the consensus check `(s - s')G == T`.

**Why it fails.**

1. **The first claim reveals `t`.** Once `s` and `s'` are public,
   `t = s - s'` is public. For every subsequent claim, anyone can
   compute `s' = s - t` from the public `s`, so the adaptor equation no
   longer proves that Bob acted. The evidence becomes a public constant.
2. **If `t` is related to `p`, the first claim compromises Bob's
   long-term receipt key** and therefore all future `b_i = p + h_i`.
3. **If `t` is independent, it does not bind the claim to Bob's receipt
   key.** It only proves knowledge of `t`, which is not the receipt
   authorization. The `ReceiptClaim` by `b_i` is still needed, so the
   adaptor adds nothing.
4. **The reusable descriptor does not help.** A static point is
   reusable, but a static adaptor secret cannot remain secret after the
   first completion. There is no way to have a reusable, non-revealed
   adaptor secret that is also publicly verifiable on-chain.

### B.4 Candidate construction 3: adaptor-separated Alice/Bob paths

**Idea.** Give Alice an exclusive per-payment secret `a_i` and Bob an
exclusive per-payment secret `b_i`. Require:

- Alice's `Plain` path to reveal `a_i` via `(s - s')G == A_i`.
- Bob's `BobClaim` path to reveal `b_i` via `(s - s')G == B_i`.

**Why it fails or requires a consensus change.**

1. **It reveals Alice's spending secret on every spend.** If `A_i` is
   reused, all of Alice's funds under that key are exposed. If `A_i` is
   per-payment, it must be bound into the output at creation, which
   changes the output format and adds an `alice` hash (the old
   `alice` field). That is a consensus change.
2. **It reveals the secret in the mempool**, creating a front-running
   vector for Alice's spend as well.
3. **It is not "existing standard Epic mechanics".** It is a new
   authorization scheme, not a `Plain` kernel with the standard
   `q_i` spend.
4. **It is not an adaptor-signature improvement.** A zero-knowledge
   Schnorr signature by `a_i` would avoid revealing the secret, but that
   is just an ordinary signature field, not an adaptor signature. The
   adaptor variant is strictly weaker on privacy.

### B.5 What is missing

A complete construction would need one of:

- **An exclusive Alice spending secret embedded in the output**, plus a
  consensus rule that requires an Alice authorization for the `Plain`
  path and a Bob authorization for the `BobClaim` path. This is a new
  key/authorization model, not an adaptor signature.
- **A script/MAST-like OR commitment** on the output, so that the
  output can be spent by either Alice's key or Bob's key without
  revealing the secret. Epic's kernel signature verifies against a
  single `excess.to_pubkey()`; there is no general script in the current
  kernel model. Adding one is a major consensus change.
- **A custom OR-proof (ring-signature-style) kernel verifier**, which
  proves knowledge of one of two secrets without revealing which one.
  This is a new signature scheme and a consensus change; it is not an
  adaptor signature.
- **A threshold/aggregate key** that only one party can complete. A
  plain 2-of-2 aggregate requires both parties, which violates the
  independent-spend requirement. A "one-of-two" aggregate is exactly
  the OR problem above.

Adaptor signatures do not supply any of these. They are a technique for
**revealing a secret on completion**, not for **OR-ing two independent
spending keys** on one Pedersen commitment.

---

## C. Authorization matrix

| Capability | Alice | Bob |
|---|---|---|
| Create Shared Output | **Yes** (using only the public descriptor) | **Not required** |
| Spend through Alice path | **Yes cryptographically, No under current consensus** | **No** (does not know Alice's exclusive secret — but see the bypass) |
| Spend through Bob path | **No** (cannot derive `b_i`) | **Yes** (`b_i`, `q_i`, `d_i`) |
| Verify Bob claim | **Yes** (`ReceiptClaim` signature + `H(B_i)` binding) | **Yes** |
| Spend Bob's final output | **No** (does not know `d_i`) | **Yes** (`d_i`) |

### C.1 How the current construction enforces these properties

- **Create Shared Output.** Alice uses `StealthReceiptDescriptor`
  (`S`, `P`) and a fresh ephemeral `r_i`; she never contacts Bob. This
  satisfies the reusable-descriptor requirement.
- **Alice path.** Cryptographically, Alice knows `q_i` and can build a
  valid `Plain` kernel with `kernel_secret = d_i - q_i`. Consensus
  rejects it through the count rule in `verify_receipt_resolution`
  (`transaction.rs:946`). This is the current conflict with §3.9.
- **Bob path.** Bob knows `b_i` and `q_i`; he builds a `BobClaim`
  kernel with a `ReceiptClaim` by `b_i`. Consensus accepts it because
  the input and kernel match 1:1 and `hashes.matches(B_i)` holds.
- **Verify Bob claim.** Alice reads the confirmed transaction, checks
  the `ReceiptClaim` signature, the `original_commitment`, the
  `kernel_excess`, the `fee`, and `H(B_i)`. She cannot forge it because
  she lacks `b_i`.
- **Spend Bob's final output.** The destination output is a normal
  `Plain` output with blind `d_i`, known only to Bob. Alice cannot spend
  it.

### C.2 The shared-secret bypass (explicit)

Alice and Bob both know `q_i`. Therefore:

- If consensus is changed to allow a `Plain` kernel to spend a
  `Receipt` input (Option A in `CURRENT_STEP.md`), **Bob can use the
  same `Plain` path**. Bob knows `q_i` and can choose `d_i`. The
  resulting transaction has no `ReceiptClaim`, so Alice cannot
  attribute it to Bob. This is a direct violation of the mandatory
  evidence requirement.
- A wallet convention ("Bob promises to use `BobClaim`") is **not** a
  cryptographic guarantee. It must not be presented as one.
- A new kernel type (`ReceiptReclaim`) does not fix the bypass by
  itself. Bob can use `ReceiptReclaim` too, because he knows `q_i`. The
  only way to make the paths exclusive is to give Alice a secret Bob
  does not know, and to make consensus require that secret on the Alice
  path.

### C.3 What would enforce separation

The minimum cryptographic requirement is:

- The output must commit to an **Alice-exclusive** public key or hash
  `A_i` (or a script/MAST root), in addition to `H(B_i)`.
- The Alice path must carry an **Alice authorization** (e.g. a Schnorr
  signature by `a_i`) that Bob cannot produce.
- The Bob path must carry the **Bob authorization** (`ReceiptClaim` by
  `b_i`) that Alice cannot produce.
- Consensus must bind each path to the correct authorization and reject
  a path with the wrong one.

This is a new authorization scheme. It is feasible, but it is a
consensus change and it changes the output format. It is **not**
provided by adaptor signatures.

---

## D. Mandatory evidence

### D.1 Current evidence is already mandatory for the BobClaim path

Under the current count rule, every `Receipt` input must have exactly
one `BobClaim` kernel (`transaction.rs:946`), and every `BobClaim`
kernel must carry a valid `ReceiptClaim` (`transaction.rs:488-504`).
The `ReceiptClaim` is a Schnorr signature by `b_i` over

```
SHA256(DOMAIN || BobClaim_tag || original_commitment ||
       kernel_excess || fee_le)
```

with `receipt_public_key = B_i`, and `verify_receipt_resolution` checks
`H(B_i) == input.receipt_key_hashes.bob`.

Cryptographic argument:

- **Bob must use his exclusive secret.** Producing a valid `ReceiptClaim`
  requires `b_i`. Under the discrete-log assumption and the ECDH
  derivation, Alice cannot compute `b_i` from `B_i`, `P`, `S`, `R_i`,
  or any public data.
- **The claim produces a valid signature.** The kernel signature and the
  `ReceiptClaim` are both valid Schnorr signatures under the respective
  keys.
- **Alice can verify.** She checks the signature and the hash binding
  from confirmed chain data.
- **Bob cannot create an equivalent valid claim without the evidence.**
  The count rule and the evidence rule force a `BobClaim` kernel with a
  valid `ReceiptClaim`. There is no `Plain` alternative while the count
  rule is in force.
- **Alice cannot produce the evidence.** She lacks `b_i`. The
  `spend_attribution.rs` forged-`BobClaim` experiment confirms rejection
  (`spend_attribution.rs:376-486`).

This is a cryptographic argument, not a wallet convention. It is the
strongest mandatory-evidence property currently available.

### D.2 Why adaptor signatures do not strengthen it

The adaptor variant would add a pre-signature `s'` and an adaptor point
`T_i`. The only consensus-checkable relation is

```
(s - s')G == T_i.
```

As shown in §B.2, this relation is satisfied by **any** valid signature
`s` once Bob knows `t_i = dlog(T_i)`, by setting `s' = s - t_i`. It does
not prove that Bob used a particular nonce, that he prepared the
pre-signature before the payment, or that he followed an adaptor
protocol. It only proves that he knows `t_i`.

If `T_i = B_i`, then `t_i = b_i` and the relation is equivalent to
"reveal `b_i`". The existing `ReceiptClaim` already proves knowledge of
`b_i` without revealing it, so the adaptor variant is redundant and
strictly less private.

If `T_i` is independent, then `t_i` is not bound to the receipt key and
the relation does not prove Bob's receipt authorization. The
`ReceiptClaim` is still needed, so the adaptor adds nothing.

If the pre-signature is **not** consensus-enforced, Bob can simply use a
normal signature and omit it. The evidence is then optional, which
violates the mandatory requirement.

### D.3 Which public signature bytes Alice would inspect

If the adaptor construction were implemented:

- Kernel signature (64 bytes): `R*.x` in bytes `0..32`, `s` in bytes
  `32..64`.
- New pre-signature field: `s'` (32 bytes).
- `ReceiptClaim.receipt_public_key`: `B_i` (33 bytes compressed).
- `Input.receipt_key_hashes.bob`: `H(B_i)` (32 bytes).

Alice would compute `t_i = s - s'` and check `t_i G == B_i` and
`H(B_i) == hashes.bob`. But `s`, `s'`, and `B_i` are all public, so
**every observer** can perform the same computation and learn `b_i`.
This is the opposite of private verification.

### D.4 Aggregation and kernel transformations

- In the current codebase, kernel signatures are **not** aggregated
  into a single signature. `aggregate` concatenates kernels
  (`transaction.rs:1284-1334`), and `batch_sig_verify` verifies each
  kernel signature individually (`transaction.rs:507-532`). So a
  per-kernel adaptor pre-signature would remain separable in the
  current code.
- If a future scheme used `aggsig::add_signatures_single` to aggregate
  kernel signatures, the individual `s` values would be combined and
  the extraction `t = s - s'` would no longer work. The vendored
  `add_signatures_single` is currently used only in a multisig helper
  (`core/src/libtx/aggsig.rs:359`), not for kernel aggregation.
- Kernel transformations (offset changes, excess re-splitting) change
  the kernel excess and therefore the `ReceiptClaim` message. The
  `ReceiptClaim` would have to be recomputed. An adaptor pre-signature
  would also have to be recomputed, because `e` depends on the excess
  public key `X`. This makes adaptor evidence fragile under any
  transformation that the wallet might apply.

### D.5 Conclusion

Adaptor signatures cannot be used to claim mandatory attribution beyond
what `ReceiptClaim` already provides. The mandatory property is a
**consensus rule**, not a signature technique. The only way to keep it
while allowing Alice a `Plain` path is to add an Alice-exclusive
authorization and a consensus rule that distinguishes the paths.

---

## E. Privacy analysis

### E.1 Relationship-by-relationship

| Relationship | Public chain observer | Pre-aggregation observer | Alice | Bob |
|---|---|---|---|---|
| Descriptor `(S, P)` → individual Shared Output `C_i` | **Not linkable** without `s` (ECDH + domain-separated hash). The output is marked `Receipt`, but the descriptor is not identifiable. | Same. | **Linkable** (she chose `r_i`). | **Linkable** (he has `s`). |
| Multiple Shared Outputs → same recipient | **Not linkable** without `s`; each `R_i` and `H(B_i)` is independent. | Same. | **Linkable** for the payments she created. | **Linkable** for all his payments. |
| Shared Output → `BobClaim` kernel | **Public**: the claim input is the `Receipt` commitment, and the kernel's `original_commitment` matches it. | Same. | Public. | Public. |
| `BobClaim` kernel → Bob destination output | **Public at the transaction level**: a 1-in-1-out claim tx contains exactly one `Plain` output. The kernel does not commit to the destination, but the tx body does. | Same. | Public. | Public. |
| Shared Output → confirmed spending transaction | **Public**: UTXO spend. | Same. | Public. | Public. |
| Bob destination output → Bob's identity | **Not public** unless the destination address is known or reused. | Same. | Not known unless Alice learns it. | Known. |
| Adaptor evidence `(s, s')` → `b_i` | **Public**: `b_i = s - s'` is computable by anyone. | Same. | Learns `b_i` (but does not need it). | Reveals his own `b_i`. |

### E.2 What the current design hides

- The descriptor-to-output link is hidden from external observers by the
  ECDH shared secret. An observer who knows `(S, P)` sees `R_i` and
  `H(B_i)` but cannot compute `h_i` or `q_i` without `s` or `r_i`.
- The recipient identity is hidden from external observers. Only Bob
  (with `s`) and Alice (with `r_i`) can link an output to the
  descriptor.
- The amount is hidden by the Pedersen commitment and Bulletproof. The
  rewind nonce is derived from `q_i`, so only Alice and Bob can rewind
  the amount.

### E.3 What the current design exposes

- The `Receipt` output feature is a public marker that the output is a
  SNIP payment. This is a protocol-level distinguisher.
- The `BobClaim` kernel feature is a public marker that the spend is a
  receipt claim.
- The claim transaction graph links the `Receipt` input to the single
  `Plain` destination output. The kernel does not reference the
  destination, but the transaction body does. This is the
  "`BobClaim` → destination" link.
- The `ReceiptClaim` reveals `B_i` and the signature. Combined with
  `H(B_i)` in the input, this links the claim to the output (which is
  necessary for attribution).
- The destination output is a normal `Plain` output. If Bob later
  spends it, the spend is a normal `Plain` spend; an observer can follow
  the graph from the claim to the later spend, but cannot link it to
  Bob's identity without additional information.

### E.4 What the adaptor construction would add

- **Public disclosure of `b_i`.** As soon as a claim with `(s, s')` is
  broadcast, anyone can compute `b_i = s - s'`. This is a new public
  secret. It does not directly reveal `p` or `q_i`, but it does reveal
  the one-time receipt secret for that output.
- **Front-running.** An observer who sees the claim in the mempool can
  compute `b_i` and build a competing `BobClaim` (or a competing
  `Plain` spend if allowed) with a higher fee. First-confirmed-spend-wins
  makes this a real risk.
- **Linkability.** The revealed `b_i` does not directly link multiple
  payments to the same recipient, because `b_i = p + h_i` and `p` stays
  secret. But it is an unnecessary secret disclosure and it makes the
  claim's evidence public rather than private.
- **Static `T` variant.** If a static `T` is used, the first claim
  reveals `t`. If `t` is related to `p`, all future receipt keys are
  compromised. If `t` is independent, the evidence is no longer
  Bob-specific after the first claim.

### E.5 Conclusion

The current design has reasonable unlinkability against external
observers for descriptor-to-output and output-to-output links, but the
claim-to-destination link is visible at the transaction level, and the
`Receipt`/`BobClaim` markers are public. The adaptor construction is
**privacy-negative**: it turns a private authorization into a public
secret and introduces a front-running vector. Ordinary-looking
signatures do not automatically guarantee unlinkability; here the
adaptor pre-signature actively creates a new link and a new secret leak.

---

## F. Required implementation changes

The following is the minimum change set **if** the adaptor construction
were pursued. It is listed for completeness; the recommendation in §G is
not to pursue it in this form.

### F.1 Cryptographic library

- **`epic_util` / vendored `grin_secp256k1zkp`:** no mandatory C change.
  The existing `sign_single`, `verify_single`, `add_signatures_single`,
  `subtract_partial_signature`, and key arithmetic are sufficient for a
  Rust-level adaptor module.
- **`epic_core` / `core/src/libtx/aggsig.rs`:** add thin wrappers for
  pre-signature creation, verification, completion, and extraction, or
  add a new `core/src/libtx/adaptor.rs`. This is a local Rust change.
- If a high-level, audited API is desired, the vendored
  `grin_secp256k1zkp` crate would need changes, but it is an external
  git dependency (`upstream/epic/util/Cargo.toml:35-38`), not part of
  this repository.

### F.2 Transaction construction

- **`core/src/receipt_claim.rs`:**
  - Add an adaptor point and pre-signature field to `ReceiptClaim` or
    `TxKernel`.
  - Extend `ReceiptClaim::message` to bind the adaptor point and
    pre-signature.
  - Add `sign_adaptor` / `verify_adaptor` / `complete` / `extract`
    functions.
  - Extend `build_bob_claim_transaction` to produce the adaptor
    variant.
- **`core/src/libtx/build.rs`:** extend `receipt_output` if the output
  must commit to the adaptor point.
- **`epic_keychain`:** derive and store the adaptor point/pre-signature;
  wallet key management for `t_i` or `b_i`.
- **Wallet integration:** descriptor publication, scanning, claim
  construction, destination selection, evidence verification, and
  front-running protection.

### F.3 Serialization

- **`core/src/core/transaction.rs`:**
  - `TxKernel` serialization (`:342-374`) must write/read the new
    fields for `BobClaim`.
  - `Input`/`Output`/`OutputIdentifier` serialization if the output
    commits to an adaptor point.
  - Any new field changes the hash of the kernel/input/output and
    therefore the MMR, the UTXO set, and historical validation.
- A protocol-version gate is required. The current receipt feature has
  no height-based activation; it uses
  `global::is_user_testing_mode()` (`transaction.rs:887-894`). A new
  field needs a proper version/activation rule.

### F.4 Consensus validation

- **`core/src/core/transaction.rs`:**
  - `verify_receipt_claim` (`:488-504`) must check the adaptor equation
    `(s - s')G == T_i`.
  - `verify_receipt_resolution` (`:897-976`) must bind `T_i` to the
    input's `H(B_i)` and enforce the new rule.
  - If the Alice path is enabled, the count rule (`:946`) and the
    metadata checks must distinguish `Plain`/Alice-authorization from
    `BobClaim`.
- **`epic_chain`:** block and txhashset validation, reorg handling, and
  historical validation. If the rule is per-transaction and the block
  body is validated after aggregation/cut-through, the cut-through
  interaction in §F.6 must be handled.

### F.5 What can remain unchanged

- **Pedersen commitment:** unchanged. `C_i = v_i G + q_i H` is a
  standard commitment.
- **Bulletproof:** unchanged. `build_shared_receipt_output` uses the
  standard `bullet_proof` and `validate` uses the standard
  `batch_verify_proofs`.
- **`ReceiptStealthHint` and `SharedReceiptKeyHashes`:** unchanged for
  the current Bob path; they can be reused.
- **ECDH derivations:** unchanged.
- **The existing `ReceiptClaim` signature:** unchanged if the adaptor
  evidence is added alongside it; it remains the actual authorization.

### F.6 Aggregation, cut-through, and historical validation

- `aggregate` (`transaction.rs:1284-1334`) concatenates inputs,
  outputs, and kernels, then calls `cut_through`
  (`transaction.rs:1250-1281`). Cut-through matches `Input::hash()` with
  `Output::hash()`; both hashes include the receipt metadata.
- If a `Receipt` output and its spending input appear in the **same
  block**, cut-through removes both and leaves the `BobClaim` kernel.
  `verify_receipt_resolution` then sees `0` receipt inputs and `1`
  receipt kernel and rejects the block (`:946`). In practice a claim
  must wait for the funding output to be confirmed, so they are in
  different blocks, but a malicious miner could try to include both.
  The consensus rule should either forbid a claim in the same block as
  its funding or make the resolution check cut-through-aware.
- Adding fields to `Input`/`Output`/`TxKernel` changes their hashes and
  therefore the MMR and the UTXO set. Historical validation of existing
  Usernet blocks would require re-indexing. This is acceptable for a
  local research chain but must be explicit.
- The `is_user_testing_mode()` gate is process-global, not a block-height
  rule. A node that switches chain type would validate historical blocks
  differently. This is a consensus/activation risk.

### F.7 Change classification summary

| Layer | Change | Classification |
|---|---|---|
| Crypto library (C) | None strictly required | `SOLVED_BY_EXISTING_CODE` |
| Crypto library (Rust adaptor module) | New wrappers over AgSig | `FEASIBLE_WITH_LOCAL_CHANGES` |
| Transaction construction | New fields, sign/verify helpers | `REQUIRES_CONSENSUS_CHANGE` if serialized |
| Serialization | New `TxKernel`/`ReceiptClaim` fields | `REQUIRES_CONSENSUS_CHANGE` |
| Consensus validation | Adaptor equation + path binding | `REQUIRES_CONSENSUS_CHANGE` |
| Alice path separation | Exclusive Alice authorization | `REQUIRES_CONSENSUS_CHANGE` |
| Wallet | Descriptor, scan, claim, evidence, front-run protection | `FEASIBLE_WITH_LOCAL_CHANGES` |
| Pedersen/Bulletproof | Unchanged | `SOLVED_BY_EXISTING_CODE` |

---

## G. Blockers and risks

### G.1 Classification

#### SOLVED_BY_EXISTING_CODE

- Pedersen commitment and Bulletproof for the Shared Output.
- ECDH + domain-separated derivation of `q_i` and `b_i`.
- `B_i = P + h_i G` and the `H(B_i)` binding.
- Schnorr AgSig sign/verify, key arithmetic, batch verification.
- `BobClaim` kernel feature and `ReceiptClaim` signature.
- 1:1 `Receipt` input ↔ `BobClaim` kernel matching.
- Reusable descriptor and Bob's chain scan.
- Existing `Plain` spend rejection for `Receipt` inputs.
- Existing tests for roundtrip, attribution, and forged claims.

#### FEASIBLE_WITH_LOCAL_CHANGES

- A Rust-level Schnorr adaptor module (pre-sign, verify, complete,
  extract) built on `sign_single`/`verify_single` and key arithmetic.
- Wallet-side adaptor pre-signature generation/verification.
- Tests for the adaptor equations, including nonce normalization.
- A Step 2 PoC that demonstrates the equations and the failure modes
  without touching consensus.

#### REQUIRES_CONSENSUS_CHANGE

- Making adaptor evidence mandatory in the transaction (new serialized
  fields + validation rule).
- Allowing Alice's `Plain` path while preventing Bob's bypass (new
  exclusive Alice authorization, new kernel feature or input metadata,
  or a script/MAST-like commitment).
- Any change to `TxKernel`/`Input`/`Output`/`ReceiptClaim` serialization.
- Any change to `verify_receipt_resolution`, `verify_receipt_claim`, or
  the activation gate.
- Re-indexing the MMR/UTXO set and handling cut-through/aggregation.

#### UNRESOLVED_CRYPTOGRAPHIC_PROBLEM

- **One Pedersen commitment with one shared blinding secret cannot
  express an OR of two independent spending keys.** Alice and Bob both
  know `q_i`. Any rule that lets Alice spend with `q_i` also lets Bob
  spend with `q_i`. There is no adaptor-signature construction that
  separates them without an exclusive secret per party and a
  consensus-visible authorization.
- **Adaptor signatures reveal the adaptor secret on completion.** If the
  secret is `b_i`, the claim is front-runnable and the evidence is
  public. If the secret is independent, it does not bind the claim to
  Bob's receipt key. If the pre-signature is not consensus-enforced,
  Bob can omit it. There is no construction that gives private,
  non-revealing, mandatory evidence better than the existing
  `ReceiptClaim`.
- **A reusable descriptor cannot provide a per-payment pre-signature in
  advance.** A Schnorr pre-signature is message- and nonce-specific. A
  pre-signature produced at claim time is not a pre-commitment and adds
  no security.
- **Therefore the combination "reusable descriptor + no Bob interaction
  per payment + private mandatory evidence + Alice independent `Plain`
  path" is not achievable with the current key model and adaptor
  signatures alone.**

### G.2 Additional risks

| Risk | Impact | Mitigation |
|---|---|---|
| Front-running if `b_i` is revealed | An observer can compete for the same `Receipt` output with a higher fee | Do not reveal `b_i`; keep the existing `ReceiptClaim` |
| `Receipt`/`BobClaim` public markers | Observers can identify SNIP outputs and claims | Accept for the research prototype; document it; consider a future privacy design |
| `is_user_testing_mode()` global gate | Historical validation depends on process state, not block height | Use a height-based activation for any real deployment |
| Cut-through removes same-block funding/claim pair | Orphan `BobClaim` kernel fails `verify_receipt_resolution` | Forbid same-block claim or make the check cut-through-aware |
| New fields change hashes/MMR | Re-indexing and historical validation issues | Version the format and re-index the Usernet chain |
| `ReceiptClaim` challenge omits `B_i` | Weaker key-prefixing than the kernel signature | Include `B_i` in the challenge for defense-in-depth |
| `ReceiptClaim` does not bind the destination output | Destination is only bound via the kernel excess and tx graph | Document the link; consider committing to the destination if privacy requires it |
| Kernel signature aggregation in a future scheme | Adaptor extraction would fail | Do not aggregate kernel signatures if adaptor evidence is required |
| No adaptor API in the vendored crate | Implementation effort and audit risk | Build a small, well-tested Rust module; do not change the C library |

### G.3 Bottom line

The existing `Receipt`/`BobClaim` design already satisfies the mandatory
evidence requirement for the Bob path, with a cryptographic argument
based on the ECDH-derived `b_i` and the `ReceiptClaim` signature. The
unresolved problem is the **Alice path**: the shared blind `q_i` makes
Alice and Bob cryptographically indistinguishable as ordinary spenders.
Adaptor signatures do not fix this and introduce a new secret leak. The
correct next step is not to implement adaptor signatures, but to decide
whether to (a) keep the count rule and drop the independent Alice
`Plain` path, or (b) add an exclusive Alice authorization and a
consensus rule that distinguishes the paths. Both options are policy
decisions with different trade-offs; neither is an adaptor-signature
problem.

---

## H. Step 2 proposal

### H.1 Goal

Validate the adaptor equations and the reusable-descriptor flow in
isolation, **without** consensus, chain, or serialization changes. The
PoC must also demonstrate the failure modes so that the architectural
decision is based on evidence, not assumption.

### H.2 Scope

- New code only:
  - `upstream/epic/core/src/libtx/adaptor.rs` (new module, not wired
    into consensus), or an equivalent test-only module.
  - `upstream/epic/core/tests/adaptor.rs` or
    `upstream/epic/core/examples/adaptor_poc.rs`.
  - Optional top-level POC additions in `src/crypto.rs` if the
    standalone crate is the preferred test bed.
- Do **not** modify `core/src/core/transaction.rs`,
  `core/src/receipt_claim.rs` consensus paths, `chain/`, or any wallet
  code.
- Do **not** commit.

### H.3 Required test cases

1. **Basic adaptor equation.**
   - Generate `x`, `X = xG`, `t`, `T = tG`.
   - Choose `k`, `R = kG`, `R* = R + T`.
   - `e = H(R*.x || X || m)`.
   - `s' = k + e*x`.
   - `s = s' + t`.
   - Assert `s'G + T == R* + eX` and `sG == R* + eX`.
   - Assert `verify_single` accepts `(R*.x, s)` under `X`.
   - Assert `t == s - s'`.

2. **Receipt-key adaptor.**
   - Use the existing ECDH derivation to obtain `b_i`, `B_i`, `q_i`.
   - Set `T = B_i`, `t = b_i`.
   - Build the adaptor pre-signature and completed signature as above.
   - Assert `H(B_i) == SharedReceiptKeyHashes::from_public_key(B_i).bob`.
   - Assert `(s - s')G == B_i`.

3. **Reusable descriptor.**
   - For three independent ephemeral keys `r_1, r_2, r_3`, derive
     `B_i` and `q_i` on Alice's side and on Bob's side.
   - Assert equality for each payment.
   - Assert the three `B_i` and `H(B_i)` values are distinct.
   - Assert an observer without `s` cannot compute `h_i` or `q_i`
     (negative test: the ECDH input is unavailable; document the
     assumption rather than trying to prove it computationally).

4. **Shared-secret bypass demonstration.**
   - Show that Alice can build a valid `Plain` kernel for a `Receipt`
     input using `q_i` and a destination blind.
   - Show that Bob can build the same `Plain` kernel.
   - Show that the current `verify_receipt_resolution` rejects both
     (count rule).
   - This is the blocker demonstration: the cryptographic ability is
     symmetric; only consensus separates the paths.

5. **Front-running demonstration.**
   - Given a completed adaptor claim `(s, s')` and `B_i`, compute
     `b_i = s - s'`.
   - Build a competing `BobClaim` (or `Plain` spend, if allowed) using
     the extracted `b_i`.
   - Show that the competing transaction is constructible and valid
     under the relevant rules.
   - This demonstrates the privacy/security regression of revealing
     `b_i`.

6. **Nonce normalization.**
   - Test cases where `R*` has non-quadratic `y`.
   - Verify that the signer retries or negates the nonce so that the
     completed signature satisfies the Epic verifier's quad-y
     convention.
   - Assert that the pre-signature equation still holds.

7. **Negative tests.**
   - Wrong adaptor point `T`.
   - Wrong message `m`.
   - Wrong nonce `k`.
   - Tampered `s'`.
   - Tampered `s`.
   - `s'` from a different payment.
   - `B_i` from a different output.
   - All must fail the relevant equation or `verify_single`.

### H.4 Expected results

- The basic adaptor equations hold with the existing AgSig primitives.
- The receipt-key adaptor works with `T = B_i` and `t = b_i`.
- The reusable descriptor derivations match for Alice and Bob.
- The Alice/Bob `Plain` spend ability is symmetric (the bypass is real).
- The adaptor evidence reveals `b_i` and enables a competing claim.
- The nonce normalization is manageable but must be handled explicitly.

### H.5 Failure conditions

The PoC is considered a **negative** result if any of the following
occurs:

- The adaptor equation cannot be made to hold with the Epic verifier's
  quad-y convention.
- The extracted `t` does not match `T` for the receipt-key case.
- The completed signature fails `verify_single`.
- The reusable descriptor derivations differ between Alice and Bob.
- An external observer can link the three outputs to the descriptor
  without `s` (which would indicate a flaw in the ECDH/hash
  construction).
- The competing claim built from the extracted `b_i` is accepted by
  `TransactionBody::validate` in a configuration that is supposed to
  reject it (which would be a serious finding).
- The PoC requires modifying consensus or serialization code (which
  would violate the Step 2 scope).

### H.6 Files that would be changed

- **New:** `upstream/epic/core/src/libtx/adaptor.rs`
  (test-only module, not referenced by `verify_receipt_claim` or
  `verify_receipt_resolution`).
- **New:** `upstream/epic/core/tests/adaptor.rs` or
  `upstream/epic/core/examples/adaptor_poc.rs`.
- **Optional:** `src/crypto.rs` / `tests/crypto_primitives.rs` in the
  top-level POC, if the standalone crate is the preferred test bed.
- **No changes:** `upstream/epic/core/src/core/transaction.rs`,
  `upstream/epic/core/src/receipt_claim.rs` consensus paths,
  `upstream/epic/chain/`, `upstream/epic/keychain/`,
  `upstream/epic/util/` (unless a thin wrapper is needed; prefer the
  new module).

### H.7 Decision gate after Step 2

Step 2 is a **cryptographic** PoC only. It does not decide the protocol
architecture. After Step 2, the project must choose between:

- **Option A:** Keep the count rule; Alice cannot independently
  `Plain`-spend; document the deviation from §3.9.
- **Option B:** Add an exclusive Alice authorization (e.g. an `alice`
  hash + Alice signature) and a consensus rule that distinguishes the
  paths; this preserves §3.9 and §3.8 but is a new consensus feature.
- **Option C:** Add a script/MAST-like OR commitment or a custom
  OR-proof kernel verifier; the most general solution, but the largest
  consensus change.

Adaptor signatures are **not** a fourth option: they do not provide an
OR of independent keys and they reveal their secret on completion.

---

## Appendix: Source map

| Topic | File:lines |
|---|---|
| `OutputFeatures::Receipt` | `upstream/epic/core/src/core/transaction.rs:1510-1522` |
| `Output` receipt fields | `upstream/epic/core/src/core/transaction.rs:1543-1563` |
| `Output` serialization | `upstream/epic/core/src/core/transaction.rs:1578-1597` |
| `Input` receipt fields | `upstream/epic/core/src/core/transaction.rs:1400-1472` |
| `OutputIdentifier` receipt fields | `upstream/epic/core/src/core/transaction.rs:1696-1706, 1767-1780` |
| `KernelFeatures::BobClaim` | `upstream/epic/core/src/core/transaction.rs:41-58` |
| `kernel_sig_msg` | `upstream/epic/core/src/core/transaction.rs:100-113` |
| `TxKernel` + `receipt_claim` | `upstream/epic/core/src/core/transaction.rs:312-374` |
| `verify_receipt_claim` | `upstream/epic/core/src/core/transaction.rs:488-504` |
| `batch_sig_verify` | `upstream/epic/core/src/core/transaction.rs:507-532` |
| `verify_receipt_feature_activation` | `upstream/epic/core/src/core/transaction.rs:887-894` |
| `verify_receipt_resolution` | `upstream/epic/core/src/core/transaction.rs:897-976` |
| `validate_read` / `validate` | `upstream/epic/core/src/core/transaction.rs:982-1012` |
| `cut_through` / `aggregate` | `upstream/epic/core/src/core/transaction.rs:1250-1334` |
| `ReceiptStealthHint` | `upstream/epic/core/src/receipt_claim.rs:28-98` |
| `StealthReceiptDescriptor` / `Secrets` | `upstream/epic/core/src/receipt_claim.rs:100-175` |
| `derive_for_sender` | `upstream/epic/core/src/receipt_claim.rs:180-200` |
| `derive_receipt_key` | `upstream/epic/core/src/receipt_claim.rs:132-151` |
| `derive_shared_spend_blind` | `upstream/epic/core/src/receipt_claim.rs:160-174` |
| `derive_shared_spend_blind_for_sender` | `upstream/epic/core/src/receipt_claim.rs:209-223` |
| `receipt_tweak` | `upstream/epic/core/src/receipt_claim.rs:226-247` |
| `shared_spend_blind` | `upstream/epic/core/src/receipt_claim.rs:253-274` |
| `build_shared_receipt_output` | `upstream/epic/core/src/receipt_claim.rs:282-308` |
| `rewind_shared_receipt_output` | `upstream/epic/core/src/receipt_claim.rs:313-326` |
| `build_bob_claim_transaction` | `upstream/epic/core/src/receipt_claim.rs:334-425` |
| `build_receipt_funding_transaction` | `upstream/epic/core/src/receipt_claim.rs:434-500` |
| `shared_rewind_nonce` | `upstream/epic/core/src/receipt_claim.rs:502-519` |
| `SharedReceiptKeyHashes` | `upstream/epic/core/src/receipt_claim.rs:521-593` |
| `ReceiptPath` / `ReceiptClaim` | `upstream/epic/core/src/receipt_claim.rs:595-745` |
| `ReceiptClaim` tests | `upstream/epic/core/src/receipt_claim.rs:747-894` |
| `receipt_output` helper | `upstream/epic/core/src/libtx/build.rs:162-...` |
| AgSig wrappers | `upstream/epic/core/src/libtx/aggsig.rs` |
| Vendored AgSig C | `depend/secp256k1-zkp/src/modules/aggsig/main_impl.h` |
| Vendored AgSig Rust | `src/aggsig.rs:88-401` |
| Key arithmetic | `src/key.rs:101-286` |
| Signature 64-byte format | `src/ffi.rs:92`, `src/constants.rs:36` |
| Chain roundtrip test | `upstream/epic/chain/tests/receipt_claim_roundtrip.rs` |
| Attribution tests | `upstream/epic/chain/tests/spend_attribution.rs` |
| Top-level POC crypto | `src/crypto.rs`, `src/one_sided.rs` |
