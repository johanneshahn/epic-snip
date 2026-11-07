# SNIP Spend Authorization Design

**Status:** Step 2 design and feasibility study. No implementation, no consensus
change, no commit.

**Scope:** Design a cryptographically enforceable separation between Alice's
and Bob's spending capabilities for the existing experimental `Receipt` output,
so that Alice can independently spend an unclaimed receipt with a `Plain`
kernel while Bob cannot use that path to bypass the mandatory `BobClaim`
evidence.

**Authoritative requirements:** [`PROTOCOL.md`](../PROTOCOL.md) §2, §3, §5.
**Prior art:** [`docs/SNIP_ADAPTOR_FEASIBILITY.md`](SNIP_ADAPTOR_FEASIBILITY.md)
(Step 1), [`docs/receipt-claim-design.md`](receipt-claim-design.md).

> **Instruction caveat.** The incoming Step 2 instruction was truncated at
> "Bob cannot use". The requirements used here were reconstructed from the
> visible text, `PROTOCOL.md`, the Step 1 report, and the existing
> `Receipt`/`BobClaim` implementation. The reconstructed requirements are
> listed in §3 and must be confirmed before any Step 3 implementation.

---

## 1. Executive summary

The shared Pedersen blind `q_i` is a **balance-construction capability**, not
an **authorization**. Alice and Bob both need `q_i` to build a valid
Mimblewimble transaction that spends the receipt output; that is unavoidable
if Bob is to claim without Alice being online. The current consensus rule
conflates the two by requiring every `Receipt` input to be resolved by a
`BobClaim` kernel, which is why Alice cannot spend at all.

Relaxing the rule to allow bare `Plain` spends is unsafe: Bob knows `q_i`, so
he can build the same `Plain` spend and omit the `ReceiptClaim` evidence.

**Recommendation:** keep `q_i` shared, and add an **independent, per-payment
Alice authorization** to the `Receipt` input. The kernel remains a standard
`KernelFeatures::Plain { fee }` kernel with the ordinary excess signature; the
extra authorization is carried by the already-experimental `Receipt` input
metadata. Consensus then enforces a two-branch predicate:

- **Bob branch:** a `BobClaim` kernel carrying a `ReceiptClaim` signed by the
  one-time receipt key `b_i` (existing mechanism, unchanged).
- **Alice branch:** a `Plain` kernel whose receipt input carries a valid
  `ReceiptSpendAuthorization` signed by Alice's one-time key `a_i`.

Bob does not know `a_i`; Alice does not know `b_i`. Neither can use the other's
branch. The shared `q_i` only lets a party *construct the balance*; it no
longer authorizes a spend.

This is a **consensus change**, not a wallet-only change. It requires:

1. `SharedReceiptKeyHashes` gains an `alice` hash (output and input metadata).
2. `ReceiptPath` gains `AliceSpend`.
3. `Input` gains an optional `ReceiptSpendAuthorization` (only for receipt
   inputs).
4. `verify_receipt_resolution` accepts exactly one of the two branches per
   receipt input.
5. A full-validation pass verifies the Alice signature.
6. Cut-through is made explicitly receipt-aware.
7. The process-global `is_user_testing_mode()` gate is replaced by a
   height-based activation for any non-test deployment.

No new kernel type is introduced, and `AliceReclaim` is not reintroduced. The
Alice path uses `KernelFeatures::Plain`, as required by `PROTOCOL.md` §3.9/§4.

---

## 2. The problem, stated precisely

Let:

- `v` be the receipt value, `q_i` the shared Pedersen blind, and
  `C_i = q_i G + v H` the receipt output commitment.
- `b_i` be Bob's one-time receipt key, derived from Bob's reusable descriptor
  and Alice's per-payment stealth hint. `B_i = b_i G` and
  `hashes.bob = H(B_i)`.
- `a_i` be Alice's fresh per-payment authorization key. `A_i = a_i G` and
  `hashes.alice = H(A_i)`.
- `q_i` be derived by ECDH between Alice's ephemeral secret and Bob's scan
  public key (or, symmetrically, Bob's scan secret and Alice's ephemeral
  public key), then expanded with a domain-separated KDF.

Knowledge matrix:

| Party | `q_i` | `b_i` | `a_i` | `C_i` opening | can build balance | can authorize Bob branch | can authorize Alice branch |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Alice | yes | no | yes | yes | yes | no | yes |
| Bob | yes | yes | no | yes | yes | yes | no |
| Mallory | no | no | no | no | no | no | no |

The current consensus rule (`TransactionBody::verify_receipt_resolution`,
`upstream/epic/core/src/core/transaction.rs:897-976`) does three things:

1. Requires `receipt_inputs.len() == receipt_kernels.len()`, where
   `receipt_kernels` are kernels with `is_receipt_resolution()` (currently only
   `KernelFeatures::BobClaim`).
2. Requires each receipt input to match exactly one `BobClaim` kernel by
   `claim.original_commitment == input.commit`.
3. Requires the input's `receipt_key_hashes` to match the claim's
   `receipt_public_key` (i.e. `hashes.bob == H(B_i)`).

Consequences:

- A `Plain` kernel spending a `Receipt` input always fails the count rule
  (`InvalidReceiptClaim`). Alice cannot spend. This is the §3.9 gap.
- If the count rule is relaxed to allow `Plain`, Bob can also build a valid
  `Plain` spend because he knows `q_i`. The spend is indistinguishable from
  Alice's, so Bob can bypass the mandatory `ReceiptClaim` evidence. This is the
  §3.8/§3.3 gap.

**Key lemma.** No consensus rule that inspects only the balance equation can
distinguish an Alice `Plain` spend from a Bob `Plain` spend. Both parties can
produce the same commitment, the same kernel excess, and a valid excess
signature. Distinguishing them requires an additional secret-based
authorization that is checked by consensus.

This lemma is the reason the solution cannot be "allow `Plain` and ask Bob to
use `BobClaim` by convention". A client-side convention is not
consensus-enforced and does not satisfy "Bob cannot bypass".

---

## 3. Security properties

The design must provide all of the following. These are the reconstructed
Step 2 requirements.

| ID | Property |
| --- | --- |
| **P1** | **Alice liveness.** Given a receipt output and Alice's `a_i`, Alice can produce a consensus-valid spend using `KernelFeatures::Plain`, without Bob. |
| **P2** | **Bob liveness.** Given a receipt output and Bob's `b_i` (derived from his reusable descriptor and the hint), Bob can produce a consensus-valid `BobClaim`, without Alice. |
| **P3** | **Bob evidence soundness.** Every consensus-valid spend of a receipt input is either (a) a `BobClaim` signed by the key whose hash is `hashes.bob`, or (b) an Alice-authorized `Plain` spend signed by the key whose hash is `hashes.alice`. Bob cannot produce (b); Alice cannot produce (a). |
| **P4** | **Alice verification.** Given the chain, Bob's descriptor, and her payment record, Alice can verify that a `BobClaim` is for her payment and is signed by the derived `b_i`. |
| **P5** | **First-confirmed-spend-wins.** The receipt output is a single UTXO; at most one of the two branches can be confirmed. |
| **P6** | **Privacy.** External observers cannot derive Alice's or Bob's identity, the value, or a deterministic link between payments. |
| **P7** | **No new kernel type.** The Alice path uses `KernelFeatures::Plain`; `AliceReclaim` is not reintroduced. |
| **P8** | **Reusable descriptor and offline payments.** Bob publishes one descriptor; Alice creates multiple payments offline; Bob prepares no per-payment signatures. |
| **P9** | **No weakened consensus.** The solution is enforced by consensus, not by wallet convention. |
| **P10** | **No forgery of evidence.** Alice cannot forge a valid `BobClaim` for a payment to Bob; Bob cannot forge a valid Alice authorization for a payment he did not receive. |

---

## 4. Threat model and trust assumptions

**Actors.**

- **Alice** — sender. Creates receipt outputs from Bob's public descriptor.
  Holds `q_i`, `a_i`, and the receipt value.
- **Bob** — recipient. Holds the descriptor secrets, derives `b_i`, scans the
  chain, and claims.
- **Mallory** — third party. Holds none of the secrets.
- **Relay / miner** — can reorder, censor, delay, or include conflicting valid
  transactions; cannot forge signatures.

**Trust assumptions.**

1. Alice obtains Bob's descriptor through an authentic channel. A
   man-in-the-middle who substitutes the descriptor can redirect payments;
   this is outside the receipt protocol and must be handled by the descriptor
   distribution mechanism.
2. Bob's `scan_secret_key` and `receipt_spend_secret_key` remain secret.
3. Alice's per-payment `a_i` remains secret until she spends.
4. ECDH on secp256k1, the hash functions, and the AgSig scheme are secure.
5. The chain's first-confirmed-spend-wins property holds within the canonical
   chain; a reorg can revert a confirmed spend (inherent).

**Out of scope.**

- Collusion between Alice and Bob. They can always produce both branches.
- Endpoint compromise, wallet malware, and side channels.
- Descriptor authenticity (flagged in §12).
- Chain-level finality beyond the canonical chain.

---

## 5. Candidate evaluation

| # | Candidate | Verdict | Reason |
| --- | --- | --- | --- |
| 1 | Bare `Plain` spend with a wallet convention | **Reject** | Bob knows `q_i` and can build the same `Plain` spend. A convention is not consensus-enforced; Bob bypasses the evidence. Violates P3, P9. |
| 2 | New `AliceReclaim` kernel type | **Reject** | Explicitly forbidden by `PROTOCOL.md` §3.9/§4 and the Step 2 scope. It also does not solve the shared-`q_i` problem by itself: Bob could use the new kernel too unless it carries an Alice signature, in which case it is equivalent to candidate 3 with a larger consensus surface. Violates P7. |
| 3 | **Exclusive Alice authorization on the `Plain` path** | **Accept (recommended)** | The kernel stays `Plain`. Consensus requires a per-payment signature by `a_i` for the Alice branch and by `b_i` for the Bob branch. Bob lacks `a_i`; Alice lacks `b_i`. Satisfies P1–P10. |
| 4 | Script / MAST-like OR commitment | **Reject for scope** | Epic has no script VM. Adding one is a much larger consensus and wallet change. Candidate 3 is the minimal special case of the same two-branch predicate (Bob key OR Alice key). It can be revisited if Epic ever gains a script layer. |
| 5 | Adaptor signatures | **Reject** | Step 1 result: completion reveals the adaptor secret; `T_i = B_i` leaks `b_i` and enables mempool front-running; a static `T = tG` becomes forgeable after the first claim; a non-consensus-enforced pre-signature can be omitted. Does not separate the paths. |
| 6 | Threshold / 2-of-2 aggregate key | **Reject** | A true 2-of-2 requires both parties for every spend, violating P1/P2. Splitting it into two independent branches collapses to candidate 3, with extra key-management complexity. |
| 7 | Exclusive `q_i` (Alice-only blind component) | **Reject** | Bob cannot claim offline without the Alice component. If Alice reveals it to Bob, anyone who sees the reveal can spend, and the reveal is either interactive (violates P8) or public (violates P3). Adaptor-based conditional reveal is circular: Bob needs the component to build the claim before the adaptor can complete. |
| 8 | Input-level vs kernel-level authorization | **Input-level recommended** | Kernel-level authorization would require a `receipt_claim` payload on a `Plain` kernel. `TxKernel::write` only serializes `receipt_claim` when `features.is_receipt_resolution()` (`transaction.rs:342-354`), so this needs either a new kernel feature (forbidden by P7) or a global kernel serialization change (changes every kernel hash and MMR). Input-level authorization is scoped to receipt inputs and leaves the kernel and its serialization untouched. |

### 5.1 Why bare `Plain` is unsafe (detail)

Bob's knowledge of `q_i` is sufficient to:

1. Choose a destination blind `r_out`.
2. Compute the kernel excess `E = (r_out - q_i) G`.
3. Sign the `Plain` kernel message with the excess secret `r_out - q_i`.
4. Produce a valid range proof for the destination output.

The resulting transaction is a perfectly valid `Plain` spend. Consensus has no
way to tell that Bob, rather than Alice, built it. Therefore any rule that
accepts a `Plain` spend of a receipt input without an additional authorization
accepts Bob's bypass.

### 5.2 Why a new kernel type is not the answer

A new `AliceReclaim` kernel would still need an Alice signature; otherwise Bob
could use it. Once it carries an Alice signature, it is functionally identical
to candidate 3, but it adds a new kernel variant, a new serialization branch,
new activation rules, and new wallet code. It also contradicts the explicit
protocol requirement that the Alice path use the existing `Plain` mechanism.

### 5.3 Why the shared blind cannot simply be split

If `q_i = q_shared + a_i` with `a_i` known only to Alice:

- Bob cannot build the balance for the claim, because he does not know `a_i`.
- If Alice sends `a_i` to Bob, the value is no longer secret from anyone who
  observes the channel or the chain, and Bob could use a `Plain` spend.
- If Alice encrypts `a_i` to Bob in the output, consensus cannot verify that
  Bob is the only one who can decrypt it, and Alice still knows `a_i`.
- A conditional reveal (adaptor) is circular: Bob needs `a_i` to construct the
  claim transaction before the adaptor can complete.

So the blind must remain shared for the offline claim to work, and the
separation must come from an independent authorization, not from the blind.

---

## 6. Recommended construction

### 6.1 Separation principle

Split the two capabilities:

- **Balance construction:** knowledge of `q_i` lets a party build a valid
  Mimblewimble transaction. Both Alice and Bob have it. It is necessary but
  **not sufficient** to spend.
- **Spend authorization:** consensus additionally requires a signature by the
  branch key:
  - Bob branch: `b_i`, whose hash is committed in `hashes.bob`.
  - Alice branch: `a_i`, whose hash is committed in `hashes.alice`.

The receipt output commits to both authorization key hashes. The spending
transaction reveals the branch key and a signature; consensus checks that the
revealed key hashes to the committed value for that branch and that the
signature verifies.

### 6.2 Keys and derivations

**Bob's reusable descriptor (unchanged).**

```
StealthReceiptDescriptor {
    scan_public_key,          // S = sG
    receipt_spend_public_key, // B = bG
}
```

Alice generates a fresh ephemeral secret `e` per payment and computes:

```
hint = eG
tweak = KDF(DOMAIN || "stealth-receipt-tweak|" || ECDH(e, S) || eG || B)
B_i = B + tweak*G
b_i = b + tweak            // Bob derives this after scanning
```

**Alice's per-payment authorization key (new).**

```
a_i = fresh random scalar, or KDF(wallet_seed || "alice-receipt-auth|" || payment_index)
A_i = a_i G
hashes.alice = H(DOMAIN || "receipt-key|alice|" || A_i)
```

`a_i` is independent of Bob's descriptor and of the ECDH shared secret. Bob
cannot derive it. It must be fresh per payment to avoid linking payments.

**Shared blind (unchanged).**

```
q_i = KDF(DOMAIN || "shared-spend-blind|" || ECDH(e, S) || eG || S)
```

Both Alice and Bob can derive `q_i`; Mallory cannot.

### 6.3 Output format (v2)

```
Output {
    features: OutputFeatures::Receipt,       // v2 semantics
    commit: q_i G + v H,
    proof: bullet proof rewound with q_i,
    receipt_key_hashes: Some(SharedReceiptKeyHashes {
        bob:   H(DOMAIN || "receipt-key|"       || B_i),  // existing derivation
        alice: H(DOMAIN || "receipt-key|alice|" || A_i),  // new, role-separated
    }),
    receipt_stealth_hint: Some(hint),
}
```

`SharedReceiptKeyHashes` grows from 32 to 64 bytes. The output identity
(`OutputIdentifier`, `transaction.rs:1696-1755`) therefore changes. This is a
consensus-visible format change and invalidates any existing v1 receipt
outputs. Because the receipt feature is Usernet-only and gated, a clean break
is acceptable for the prototype; a versioned `ReceiptV2` variant is the safer
long-term option (see §12).

The input must carry the identical `receipt_key_hashes` and hint. The UTXO
lookup already rebuilds the identifier from the input
(`utxo_view.rs:79-91`, `txhashset.rs:966`), so a mismatch is rejected.

### 6.4 Bob branch: `BobClaim` (unchanged)

```
KernelFeatures::BobClaim { fee }
receipt_claim = ReceiptClaim {
    path: ReceiptPath::BobClaim,
    original_commitment: C_i,
    kernel_excess: E,
    fee,
    receipt_public_key: B_i,
    receipt_signature: Sig_{b_i}(H(DOMAIN || path || C_i || E || fee)),
}
```

Bob builds this exactly as `build_bob_claim_transaction`
(`receipt_claim.rs:334-425`) does today. No per-payment interaction with Alice
is needed: `b_i` is derived from his descriptor and the public hint.

Consensus checks (existing, retained):

- The kernel's `receipt_path()` is `BobClaim`.
- `claim.path == BobClaim`, `claim.kernel_excess == kernel.excess`,
  `claim.fee == kernel.fee`, and `claim.verify()`.
- The matching input's `hashes.bob == H(claim.receipt_public_key)`.
- Exactly one `BobClaim` kernel matches each receipt input.

### 6.5 Alice branch: `Plain` kernel plus input authorization

The kernel is a **standard** `Plain` kernel:

```
KernelFeatures::Plain { fee }
excess = (r_out - q_i) G
excess_sig = Sig_{r_out - q_i}(H(Plain || fee))
receipt_claim = None
```

The kernel's serialization, hashing, and signature are unchanged. The extra
authorization lives on the receipt input:

```
Input {
    features: OutputFeatures::Receipt,
    commit: C_i,
    receipt_key_hashes: Some(hashes),      // must match the output
    receipt_stealth_hint: Some(hint),      // must match the output
    receipt_spend_authorization: Some(ReceiptSpendAuthorization {
        path: ReceiptPath::AliceSpend,
        original_commitment: C_i,
        kernel_excess: E,
        fee,
        kernel_hash: H(kernel),            // binds the exact Plain kernel
        alice_public_key: A_i,
        alice_signature: Sig_{a_i}(message),
    }),
}
```

`ReceiptSpendAuthorization` contains only `Copy` fields, so `Input` remains
`Copy`. It is serialized only for `features.is_receipt()` inputs, so ordinary
inputs are byte-for-byte unchanged.

The input hash includes the authorization (because `Input`'s `Writeable`
serializes it). That has two deliberate consequences:

- The authorization is committed by the input hash.
- A receipt input's hash no longer equals the corresponding output's hash, so
  the generic `cut_through` path will not remove it. This must still be made
  explicit (see §6.8) rather than relying on the serialization difference.

`OutputIdentifier::from_input` ignores the authorization, so the UTXO lookup
continues to compare only `features`, `commit`, `receipt_key_hashes`, and
`receipt_stealth_hint`. This is safe because the authorization is checked
separately by consensus.

### 6.6 Authorization message

```
message = H(
    DOMAIN || b"receipt-spend-authorization|" || version || chain_id ||
    path.tag() ||
    original_commitment ||
    kernel_excess ||
    fee_le ||
    kernel_hash ||
    alice_public_key_compressed
)
```

- `path.tag()` is `AliceSpend` (a new tag, distinct from `BobClaim`).
- `version` is the receipt feature version.
- `chain_id` prevents cross-chain replay.
- `kernel_hash` binds the exact `Plain` kernel, including its `excess_sig`, so
  an attacker cannot duplicate or substitute the kernel without invalidating
  the authorization.
- `alice_public_key` binds the signing key; the verifier also checks
  `H(A_i) == hashes.alice`.
- The message does **not** bind the destination output commitment. This mirrors
  the existing `ReceiptClaim` and preserves aggregation. It is safe because the
  excess binds the destination blinding, and a range proof binds the value: an
  attacker who does not know the destination blind cannot redirect the spend.
  Binding a destination commitment is a possible hardening (§12).

Signature format: AgSig, 64 bytes (`R.x || s`), challenge
`e = H(R.x || A_i || message)` (key-prefixed). The Alice branch should use
`pubkey_for_e = Some(&A_i)` for defense-in-depth. The existing `BobClaim` uses
`pubkey_for_e = None` (`receipt_claim.rs:677-686`); upgrading it is a separate
hardening item.

### 6.7 Consensus validation rules

The new rule replaces the count rule in `verify_receipt_resolution`
(`transaction.rs:897-976`). Let `R` be the receipt inputs and `K` the kernels.

```
for each input r in R:
    bob_matches = { k in K : k.path == BobClaim
                          and k.claim.original_commitment == r.commit }
    alice_auth  = r.receipt_spend_authorization

    if bob_matches.len() == 1 and alice_auth is None:
        require hashes(r).matches_bob(bob_matches[0].claim.receipt_public_key)
        mark bob_matches[0] as used by r

    else if bob_matches.len() == 0 and alice_auth is Some(a):
        require a.path == AliceSpend
        require a.original_commitment == r.commit
        require hashes(r).matches_alice(a.alice_public_key)
        require exactly one Plain kernel k in K with:
            k.excess == a.kernel_excess
            and k.features.fee() == a.fee
            and H(k) == a.kernel_hash
        mark k as used by r

    else:
        reject InvalidReceiptClaim        // no path, ambiguous path, or both

for each BobClaim kernel k in K:
    require exactly one r in R used by k
for each Plain kernel k bound by an Alice authorization:
    require exactly one r in R used by k
require each r in R is used exactly once
```

The signature check for `a.alice_signature` is performed in the full validation
path, mirroring the existing `BobClaim` split:

- `validate_read` performs the structural checks above (paths, hashes, 1:1
  matching, kernel binding).
- `validate` calls a new `verify_receipt_spend_authorizations()` (or extends
  `TxKernel::batch_sig_verify` with the inputs) to verify the Alice signature
  and the kernel excess signature.

`verify_receipt_feature_activation` (`transaction.rs:887-895`) must treat a
receipt input with an Alice authorization as a receipt feature (it already
does, because the input's features are `Receipt`) and reject it before the
activation height.

`ReceiptPath` gains `AliceSpend` with a distinct tag; `ReceiptPath::read`
accepts both. `KernelFeatures::receipt_path()` remains `BobClaim`-only, because
the Alice authorization is not a kernel feature. A new helper
`Input::receipt_spend_path()` can return `Some(AliceSpend)` for an authorized
receipt input.

### 6.8 Serialization, hashing, and cut-through

| Item | Change | Effect |
| --- | --- | --- |
| `SharedReceiptKeyHashes` | add `alice: [u8; 32]` | Receipt output/input identity changes; all fixtures update. |
| `ReceiptPath` | add `AliceSpend` | New tag; `ReceiptClaim` message gains the path. |
| `Input` | add `receipt_spend_authorization: Option<ReceiptSpendAuthorization>` | Serialized only for receipt inputs. Ordinary inputs unchanged. |
| `Input::hash` | includes the authorization | Commits the authorization; receipt input hash differs from output hash. |
| `OutputIdentifier::from_input` | unchanged (ignores authorization) | UTXO lookup still matches on commit + receipt metadata. |
| `cut_through` / `verify_cut_through` | skip receipt inputs and outputs | Prevents removal of the resolution evidence and the authorized input. |
| `verify_receipt_resolution` | two-branch matching | Enforces the new rule. |
| `verify_receipt_feature_activation` | height-based activation | Replaces the process-global flag for non-test deployments. |

**Cut-through must be explicit.** The current `cut_through`
(`transaction.rs:1250-1281`) removes any input whose hash equals an output
hash. If a receipt output and its `BobClaim` input are created in the same
block, cut-through removes both and leaves an orphan `BobClaim` kernel, which
then fails `verify_receipt_resolution` (0 inputs, 1 kernel). The design must
skip receipt-feature inputs and outputs in both `cut_through` and
`verify_cut_through`. Not cutting through is always valid; it only costs block
space. This is required for Bob's evidence and for Alice's authorization.

### 6.9 End-to-end flows

**Alice creates a payment (offline, no Bob).**

1. Obtain Bob's descriptor `(S, B)` authentically.
2. Generate fresh `e` and fresh `a_i`.
3. Derive `hint = eG`, `B_i = B + tweak*G`, `q_i`.
4. Compute `hashes = { bob: H(B_i), alice: H(A_i) }`.
5. Build the receipt output `C_i = q_i G + v H` with a rewound bullet proof.
6. Store `(payment_id, e, a_i, q_i, C_i, v, hashes, hint)` in Alice's wallet.
7. Submit a funding transaction that creates the receipt output.

**Bob scans (no Alice).**

1. For each `Receipt` output, read `hint`.
2. Derive `b_i` and `B_i` from his descriptor secrets.
3. Check `H(B_i) == hashes.bob`. If not, ignore.
4. Derive `q_i` and rewind the range proof to learn `v`.
5. Record the payment.

**Bob claims (no Alice).**

1. Build `build_bob_claim_transaction` as today.
2. Submit. Consensus accepts the `BobClaim` branch.

**Alice reclaims (no Bob).**

1. Look up the unspent receipt output and her stored `a_i`, `q_i`, `v`.
2. Choose a fresh destination blind `r_out` and compute
   `E = (r_out - q_i) G`.
3. Build a `Plain { fee }` kernel with excess `E` and the ordinary excess
   signature.
4. Compute `kernel_hash = H(kernel)`.
5. Build `ReceiptSpendAuthorization` with `path = AliceSpend`, `C_i`, `E`,
   `fee`, `kernel_hash`, `A_i`, and `Sig_{a_i}(message)`.
6. Attach it to the receipt input and submit. Consensus accepts the Alice
   branch.

**Alice verifies Bob's claim (from the chain).**

1. Find the `BobClaim` kernel whose `claim.original_commitment == C_i`.
2. Derive `B_i` from Bob's descriptor and the hint; check
   `claim.receipt_public_key == B_i`.
3. Check `hashes.bob == H(B_i)`.
4. Check `claim.verify()` and that the claim's `kernel_excess`/`fee` match the
   kernel.
5. Confirm the destination output belongs to Bob (Bob's wallet can prove
   ownership; the chain shows the value flowed to a Bob-controlled output).

Step 2 is a fake-evidence check: if Alice created a self-claimed receipt with a
bogus `bob` hash, step 2 fails because the derived `B_i` does not match.

### 6.10 First-confirmed-spend-wins

Both branches spend the same UTXO `C_i`. The UTXO set can remove it once. The
first transaction confirmed in the canonical chain wins; the other branch's
transaction becomes invalid (`AlreadySpent`). Mempool races are resolved by
miners; the protocol does not give Bob priority. This matches
`PROTOCOL.md` §3.9 and the pre-Step-1 correction ("no Bob priority").

---

## 7. Authorization matrix

| Spender | `q_i` | `b_i` | `a_i` | Bob branch | Alice branch | Consensus |
| --- | --- | --- | --- | --- | --- | --- |
| Alice | yes | no | yes | no | yes | Alice `Plain` accepted |
| Bob | yes | yes | no | yes | no | `BobClaim` accepted |
| Mallory | no | no | no | no | no | rejected |
| Alice + Bob colluding | yes | yes | yes | yes | yes | accepted (out of scope) |

---

## 8. Attack analysis

| Attack | Result | Why |
| --- | --- | --- |
| **Bob uses Alice's `Plain` path to bypass evidence** | Rejected | The input authorization requires `Sig_{a_i}`; Bob does not know `a_i`. The shared `q_i` only lets him build the balance, not the authorization. |
| **Alice forges Bob's `BobClaim` evidence** | Rejected | She does not know `b_i`; the claim signature and `hashes.bob` binding fail. A self-created receipt with a bogus `bob` hash is not a payment to Bob and is caught by Alice's derivation check. |
| **Third-party theft** | Rejected | Mallory knows neither `q_i` nor `a_i`/`b_i`; she cannot build the balance or either authorization. |
| **Replay of Alice's authorization** | Rejected | The message binds `C_i`, `E`, `fee`, `kernel_hash`, `A_i`, version, and chain id. The same UTXO can be spent once. |
| **Replay across chains** | Rejected | `chain_id` in the message; recommend also versioning `DOMAIN`. |
| **Front-running** | Limited | An attacker can race a conflicting spend, but cannot redirect either branch: Bob cannot use Alice's authorization (no `a_i`), and Alice cannot use Bob's claim (no `b_i`). First-confirmed wins. |
| **Malleability** | Limited | The authorization binds the exact kernel hash and excess; the excess binds the destination blind; the range proof binds the value. An attacker without the destination blind cannot redirect funds. |
| **Kernel duplication / ambiguity** | Rejected | The authorization binds `H(kernel)`, so a duplicate kernel does not match; the 1:1 matching rule rejects ambiguity. |
| **Mixed branches on one input** | Rejected | The per-input rule requires exactly one branch; both a `BobClaim` match and an Alice authorization is rejected. |
| **Collusion** | Accepted | Alice and Bob can produce both branches. Out of scope; no protocol can prevent it. |
| **Malicious miner** | Limited | A miner can censor, reorder, or choose between conflicting valid spends; it cannot forge a signature or redirect funds. |
| **Cut-through removes evidence** | Rejected by design | Receipt inputs/outputs are exempt from cut-through; otherwise a same-block `BobClaim` would orphan its kernel. |
| **Aggregation** | Supported | Kernels and inputs aggregate normally; the 1:1 matching is per body; receipt inputs/outputs are preserved by the cut-through exemption. |
| **Reorg** | Inherent | First-confirmed-spend-wins holds within the canonical chain; a reorg can revert a spend. |
| **Activation bypass** | Rejected | Receipt features are rejected before the activation height; the process-global test flag is not a production rule. |
| **Descriptor substitution (MITM)** | Out of scope, flagged | Alice must obtain Bob's descriptor authentically. A substituted descriptor redirects the payment; this is a descriptor-distribution problem. |
| **`a_i` reuse** | Privacy risk | Reusing `a_i` links payments. The wallet must generate a fresh `a_i` per payment. |
| **Signature key-prefixing** | Hardening | The Alice branch uses `pubkey_for_e = Some(&A_i)`. The existing `BobClaim` uses `None`; upgrading it is a separate hardening item. |

---

## 9. Privacy analysis

**Per relationship.**

- **Alice ↔ Bob:** Bob learns the payment value (by rewinding), the hint, and
  the receipt output. He does not learn `a_i`. Alice learns that Bob claimed
  (from the chain) and the claim's destination output, but not Bob's identity
  beyond the descriptor.
- **Alice ↔ observer:** The output reveals `H(B_i)`, `H(A_i)`, and a one-time
  hint. Without Alice's or Bob's keys, the observer cannot link these to an
  identity. The value is hidden by the Pedersen commitment and the range proof.
- **Bob ↔ observer:** The `BobClaim` reveals `B_i`, which hashes to
  `hashes.bob`. This is inherent to the existing evidence design. It does not
  reveal Bob's long-term keys, and fresh `b_i` per payment prevents
  cross-payment linkage.
- **Alice's spend ↔ observer:** The `Plain` spend reveals `A_i`, which hashes
  to `hashes.alice`. Fresh `a_i` per payment prevents cross-payment linkage.

**Linkability.**

- `b_i` is a function of Bob's descriptor and the per-payment hint. Different
  hints produce different `b_i`; no deterministic link across payments.
- `a_i` is fresh per payment; no deterministic link.
- The ephemeral hint is fresh per payment.
- The destination blind is fresh per spend.
- The only public identifiers are hashes of fresh keys. Reuse of `a_i`, `e`,
  or the destination blind is the main linkability risk and must be prevented
  by the wallet.

**Amounts.** The value is hidden in the commitment and the range proof. Bob
can rewind because he knows `q_i`; Alice knows `v`; Mallory cannot.

**Metadata.** The receipt input/output pair is visible as a receipt feature
(consensus-visible). The Alice branch reveals that a receipt was spent via the
Alice path, but not Alice's identity beyond `A_i`. The Bob branch reveals the
claim and `B_i`. This is the minimum required for the two-branch enforcement.

---

## 10. Implementation impact and classification

### 10.1 Consensus / core (`upstream/epic/core`)

- `receipt_claim.rs`:
  - `SharedReceiptKeyHashes`: add `alice`, `matches_bob`, `matches_alice`,
    update `from_public_key`, `Writeable`, `Readable`, and
    `SharedReceiptOutputIdentifier`.
  - `ReceiptPath`: add `AliceSpend`, tag, `Writeable`, `Readable`.
  - `ReceiptClaim`: unchanged fields; the message already includes the path
    tag. Consider adding `version`/`chain_id` for replay hardening.
  - New `ReceiptSpendAuthorization` struct and message/sign/verify helpers.
  - `build_shared_receipt_output`: take the Alice key hash.
  - New `build_alice_spend_transaction` helper.
  - `build_bob_claim_transaction`: unchanged.
- `core/transaction.rs`:
  - `Input`: add `receipt_spend_authorization`, update `Writeable`/`Readable`,
    keep `Copy`.
  - `OutputIdentifier::from_input`: unchanged (ignores the authorization).
  - `verify_receipt_resolution`: two-branch matching.
  - New `verify_receipt_spend_authorizations` (full validation).
  - `verify_receipt_feature_activation`: height-based activation.
  - `cut_through` / `verify_cut_through`: receipt exemption.
  - `TxKernel::verify_receipt_claim`: unchanged for `BobClaim`; ensure a
    `Plain` kernel cannot carry a `receipt_claim`.
- `core/hash.rs`: no change expected; the `Input` hash picks up the new field
  through `DefaultHashable`.

### 10.2 Chain / node (`upstream/epic/chain`)

- Thread the activation height through transaction validation.
- Keep the UTXO lookup unchanged (`OutputIdentifier::from_input`).
- Update the mempool/block validation paths that call `validate_read` /
  `validate`.
- Update `txhashset` tests and fixtures.

### 10.3 Wallet

- Alice:
  - Generate and persist `a_i` (or derive it deterministically).
  - Build the receipt output with `hashes.alice`.
  - Detect her own receipt outputs by `hashes.alice`.
  - Build the `Plain` reclaim transaction and the input authorization.
  - Verify Bob's claim by deriving `B_i` and checking `hashes.bob`.
- Bob:
  - Scan with the descriptor; check `hashes.bob` (unchanged).
  - Claim with `build_bob_claim_transaction` (unchanged).
  - Optionally record `hashes.alice` for audit, but do not use it.
- Descriptor: unchanged.

### 10.4 Tests

- Update all `Receipt`/`SharedReceiptKeyHashes` fixtures for the `alice` field.
- New consensus tests:
  - Alice `Plain` spend with a valid authorization → accepted.
  - Alice `Plain` spend without an authorization → rejected.
  - Alice `Plain` spend with a wrong-key authorization → rejected.
  - Bob `Plain` spend (copying Alice's authorization) → rejected / cannot
    redirect.
  - Bob `BobClaim` → accepted (unchanged).
  - Alice-forged `BobClaim` → rejected (unchanged).
  - Mixed branch (BobClaim + Alice authorization on one input) → rejected.
  - Duplicate/ambiguous kernel binding → rejected.
  - Cut-through with a receipt input/output in the same block → evidence
    preserved.
  - Activation height → receipt features rejected before activation.
- New chain-level acceptance tests:
  - Bob publishes a descriptor; Alice creates three payments offline; all
    three confirm.
  - Bob scans and claims all three; Alice verifies each claim.
  - Alice reclaims an unclaimed payment before Bob; Bob's later claim is
    rejected.
  - Bob attempts a `Plain` spend of an unclaimed payment; rejected.
  - Alice attempts to forge a `BobClaim`; rejected.
- Wallet tests for `a_i` persistence/recovery and fresh-key generation.

### 10.5 Documentation

- `PROTOCOL.md`: clarify that the Alice path is a `Plain` kernel with a receipt
  input authorization, and that the two branches are consensus-enforced.
- `docs/receipt-claim-design.md`: add the Alice branch and the `alice` hash.
- This document: keep as the Step 2 record.

---

## 11. Test plan (Step 3)

1. Unit tests for `SharedReceiptKeyHashes`, `ReceiptPath`, and
   `ReceiptSpendAuthorization` serialization and domain separation.
2. Unit tests for the authorization message and signature (valid, wrong key,
   wrong path, wrong commitment, wrong excess, wrong fee, wrong kernel hash,
   wrong chain id).
3. `verify_receipt_resolution` table tests for all branch combinations.
4. `verify_receipt_spend_authorizations` full-validation tests.
5. Cut-through and aggregation tests.
6. Activation-height tests.
7. Chain-level `spend_attribution` tests (updated) and a new
   `snip_acceptance` test for the full three-payment flow.
8. Wallet scan/claim/reclaim tests.
9. Regression: existing `epic_core` and `epic_chain` suites.

---

## 12. Blockers, risks, and open decisions

### Blockers

1. **Truncated Step 2 instruction.** The reconstructed requirements in §3 must
   be confirmed before Step 3.
2. **Consensus change.** The solution cannot be implemented in the wallet
   alone. It requires a fork/activation and a coordinated update of core,
   chain, and wallet.
3. **Output format change.** Adding `alice` changes the receipt output
   identity. Existing v1 receipt outputs and fixtures become invalid. In
   Usernet this is acceptable; in any persistent network it needs a versioned
   migration.
4. **Activation.** The current gate is the process-global
   `global::is_user_testing_mode()`, not a height-based consensus rule. A real
   deployment needs an activation height and a pre-activation rejection rule.

### Risks

1. **Cut-through.** Must be explicitly receipt-aware; otherwise evidence can be
   removed. The current `cut_through` is not receipt-aware.
2. **Wallet key management for `a_i`.** Random `a_i` must be backed up; a
   deterministic derivation must be recoverable. A lost `a_i` means Alice can
   no longer reclaim (but Bob can still claim).
3. **Privacy of `a_i`.** Reuse links payments. Fresh per payment is mandatory.
4. **Descriptor authenticity.** A MITM-substituted descriptor redirects the
   payment. Out of scope but must be documented.
5. **Fake self-claims.** Consensus cannot verify that `hashes.bob` was derived
   from Bob's real descriptor; Alice's verification must check the derivation.
   Third parties must also check the descriptor.
6. **Signature key-prefixing.** The existing `BobClaim` uses
   `pubkey_for_e = None`; the Alice branch should use `Some(&A_i)`. Upgrading
   `BobClaim` is a separate hardening item.
7. **Destination binding.** The authorization binds the excess, not the
   destination commitment. The excess + range proof analysis says this is safe,
   but binding a destination commitment is a possible hardening with an
   aggregation trade-off.
8. **Collusion.** Alice and Bob can always collude. The design does not claim
   to prevent it.
9. **Reorg.** First-confirmed-spend-wins is within the canonical chain.

### Open decisions for Step 3

1. Confirm the reconstructed requirements in §3.
2. Confirm the input-level authorization (recommended) vs the kernel-level
   variant (requires a new kernel feature or a global serialization change).
3. Confirm the clean `Receipt` v2 break vs a versioned `ReceiptV2` variant.
4. Confirm the activation mechanism (height-based; Usernet genesis/0).
5. Confirm `a_i` key management (random + persist vs deterministic derivation).
6. Confirm whether to bind a destination commitment in the authorization.
7. Confirm whether to upgrade `BobClaim` to key-prefixed signatures in the same
   change or separately.

---

## 13. Step 3 proposal and decision gate

**Proposed Step 3 (only after explicit approval):**

1. Add `alice` to `SharedReceiptKeyHashes` and `AliceSpend` to `ReceiptPath`.
2. Add `ReceiptSpendAuthorization` and the `Input` field.
3. Implement the two-branch `verify_receipt_resolution`, the full-validation
   signature pass, and the receipt-aware cut-through.
4. Replace the test-mode gate with a height-based activation for the receipt
   rules.
5. Add `build_alice_spend_transaction` and update `build_shared_receipt_output`.
6. Update all fixtures and tests, including the chain-level acceptance flow.
7. Run the full `epic_core` and `epic_chain` suites.
8. Do not commit until the results are reviewed.

**Decision gate:** Step 3 must not start until the user confirms §12's open
decisions and explicitly approves implementation.

---

## Appendix A: Code references

| Item | Location |
| --- | --- |
| `TxKernel`, `KernelFeatures`, `TxKernel::verify_receipt_claim` | `upstream/epic/core/src/core/transaction.rs:312-548` |
| `Input`, `Output`, `OutputFeatures` | `upstream/epic/core/src/core/transaction.rs:1400-1623` |
| `OutputIdentifier::from_input` | `upstream/epic/core/src/core/transaction.rs:1748-1755` |
| `verify_receipt_resolution` | `upstream/epic/core/src/core/transaction.rs:897-976` |
| `verify_receipt_feature_activation` | `upstream/epic/core/src/core/transaction.rs:887-895` |
| `validate_read` / `validate` | `upstream/epic/core/src/core/transaction.rs:982-1012` |
| `cut_through` | `upstream/epic/core/src/core/transaction.rs:1250-1281` |
| `verify_cut_through` | `upstream/epic/core/src/core/transaction.rs:843-860` |
| `SharedReceiptKeyHashes` | `upstream/epic/core/src/receipt_claim.rs:522-593` |
| `ReceiptPath` | `upstream/epic/core/src/receipt_claim.rs:595-622` |
| `ReceiptClaim` | `upstream/epic/core/src/receipt_claim.rs:624-745` |
| `build_shared_receipt_output` | `upstream/epic/core/src/receipt_claim.rs:282-308` |
| `build_bob_claim_transaction` | `upstream/epic/core/src/receipt_claim.rs:334-425` |
| `build_receipt_funding_transaction` | `upstream/epic/core/src/receipt_claim.rs:434-500` |
| Shared blind / receipt tweak derivations | `upstream/epic/core/src/receipt_claim.rs:226-274` |
| UTXO input lookup | `upstream/epic/chain/src/txhashset/utxo_view.rs:79-91` |
| Existing attribution experiments | `upstream/epic/chain/tests/spend_attribution.rs` |
| Existing one-payment roundtrip | `upstream/epic/chain/tests/receipt_claim_roundtrip.rs` |

## Appendix B: Why adaptor signatures do not solve this

See [`docs/SNIP_ADAPTOR_FEASIBILITY.md`](SNIP_ADAPTOR_FEASIBILITY.md). In
short:

- Completing an adaptor signature reveals the adaptor secret; if the secret is
  `b_i`, Bob's claim key is public and can be front-run.
- A static adaptor point `T = tG` becomes forgeable after the first claim.
- A pre-signature that is not consensus-enforced can simply be omitted.
- The shared `q_i` means both parties can build the balance; adaptor
  signatures do not create a consensus-visible difference between the paths.

The Step 1 verdict stands: the existing `ReceiptClaim` by `b_i` is already the
mandatory, non-forgeable evidence for the Bob branch; the missing piece is an
independent Alice authorization, not an adaptor signature.
