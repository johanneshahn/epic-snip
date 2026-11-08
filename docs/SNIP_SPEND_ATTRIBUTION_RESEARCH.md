# SNIP: Spend Attribution — Research Report

**Status: HISTORICAL — completed and superseded by
[PROTOCOL.md](../PROTOCOL.md).**

This report documents the spend-attribution research from the phase
when an `AliceReclaim` mechanism still existed. The
`AliceReclaim` mechanism was **completely removed**; the report
remains as a historical record.

**The top priority of this project today is the complete
3-payment demonstration per [PROTOCOL.md](../PROTOCOL.md) §5.**
All following content is **historical** and not guidance for further
development.

---

## Summary (historical)

**Alice can cryptographically verify that a confirmed
transaction is a valid BobClaim** (i.e., Bob authorized the spend),
using only confirmed chain data.

**Alice cannot spend the shared Receipt output herself.**
Consensus validation requires that every input with
`features = Receipt` matches one-to-one with a
receipt-resolution kernel (currently `BobClaim`). An
ordinary kernel spending a Receipt output is rejected with
`InvalidReceiptClaim`.

The attribution model is now:

- **When Bob claims:** the kernel has `features = BobClaim`, a
  `receipt_claim` is present, and `receipt_claim.receipt_public_key`
  matches `hashes.bob` (the Bob-specific hash). Alice can verify
  this from public data.
- **When anyone else attempts a spend:** consensus rejects the
  transaction. There is no "Alice-Spend" path.

## Q1: Can Alice distinguish a Bob claim from any other spend?

**Yes, for confirmed transactions.** When Alice observes a
confirmed transaction that spends her shared Receipt output:

- If `kernel.features` is `BobClaim { fee }` and
  `kernel.receipt_claim` is present, she checks
  `receipt_claim.receipt_public_key` against `hashes.bob`. On a
  match, the claim was cryptographically authorized by Bob
  (only Bob holds the `receipt_spend_secret_key` needed to
  derive `bob_receipt_key`).
- Any other kernel type (Plain, etc.) would have been
  rejected by consensus and cannot appear in a confirmed block.

**There is no other spending path.** The shared Receipt output
can only be consumed by a BobClaim. There is no Alice reclaim,
no ordinary spend, no alternative resolution path.

## Q2: What cryptographic binding ties the BobClaim specifically to Bob?

The binding chain is:

1. **Alice creates the Receipt** with
   `SharedReceiptKeyHashes { bob: H(bob_receipt_key) }`.
2. **Bob derives `bob_receipt_key`** using
   `receipt_spend_secret_key` (known only to Bob):
   `bob_receipt_key = receipt_spend_secret_key + tweak`
   (where the tweak is derived from Alice's ephemeral key).
3. **Bob's claim** contains
   `receipt_public_key = bob_receipt_key * G`.
4. **Consensus validates** (in `verify_receipt_claim`):
   `hashes.bob == receipt_key_hash(claim.receipt_public_key)`.

The decisive property: **only Bob can produce a
`receipt_public_key` whose hash equals `hashes.bob`**, because
only Bob knows `receipt_spend_secret_key`. Alice cannot forge
this without Bob's secret.

Additionally, the AgSig in `ReceiptClaim::verify()` requires
the private `bob_receipt_key` to sign
`(path, original_commitment, kernel_excess, fee)`. Even if
Alice knew `bob_receipt_public_key`, she could not produce a
valid signature.

## Q3: Can Alice forge a BobClaim (impersonate Bob)?

**No.** Alice would have to produce a
`receipt_public_key` such that
`H(receipt_public_key) == hashes.bob`. Since
`hashes.bob = H(bob_receipt_key)` and `bob_receipt_key` is
derived from `receipt_spend_secret_key` (which Alice does not
know), Alice cannot compute the correct
`receipt_public_key`.

**Important tested case (Experiment C, case 2):** Alice *knows*
`bob_receipt_public_key` (it is the output of
`derive_for_sender`). She can insert it into the
`ReceiptClaim` and the **hash check passes**. However, she
cannot produce a valid AgSig because she does not know
`bob_receipt_key` (the private key). The signature check
**fails**, and the transaction is rejected.

**Experimental verification:**
- **Case 1 (random pubkey):** hash check fails. Rejected.
- **Case 2 (correct Bob pubkey + forged signature):** hash
  check passes, AgSig fails. Rejected.
- **Case 3 (copied public fields from a previous claim):**
  signature bound to a different kernel. Rejected.
- **Case 4 (Alice's own secrets):** hash check fails.
  Rejected.
- **Case 5 (Alice-controlled destination):** defense in
  depth — even if all checks passed, the destination would be
  Alice-controlled (not observable without a valid claim).

## Q4: What happens if Bob omits `receipt_claim` from his kernel?

Consensus validation rejects the kernel because
`ReceiptClaim` is a required field for
`features = BobClaim`. Without `receipt_claim`, the
transaction is rejected with `MissingReceiptClaim`.

## Q5: Can an outsider recognize that an output was claimed by Bob?

**Yes, this is intended.** The `BobClaim` kernel is a
publicly visible special kernel. An outsider can recognize:
- that a Receipt output was spent,
- that the spend occurred via a `BobClaim` kernel,
- that the signature is valid.

**No, not what Bob received.** The claim's destination output
is an ordinary output that is not referenced in the
`BobClaim` kernel. An outsider cannot correlate the claim
transaction with Bob's later spends.

## Test Matrix

| Test | Goal | Result |
| --- | --- | --- |
| A | Valid BobClaim accepted | ✅ PASS |
| B1 | Alice spend rejected | ✅ PASS |
| B2 | Alice forgery rejected | ✅ PASS |
| B3 | Bob spend with metadata rejected | ✅ PASS |
| B4 | Bob spend without metadata rejected | ✅ PASS |
| C | 5 cases: Bob impersonation rejected | ✅ PASS |
| C-full | Full chain roundtrip + classification | ✅ PASS |
| D1 | Persistence: receipt hashes over serialization | ✅ PASS |
| D2 | Kernel structure: BobClaim transcript | ✅ PASS |
| D3 | Unforgability: hash binding | ✅ PASS |
| D4 | Unforgability: AgSig binding | ✅ PASS |
| E | Static analysis: no Alice-spend paths | ✅ PASS |
| Bonus | Full attribution chain | ✅ PASS |

**Total: 13/13 PASS.**

## Conclusion (historical)

The spend-attribution model is **valid**. The BobClaim
mechanism is unforgable, there is no bypass path, and
attribution is verifiable from public chain data.

## Transition to the New SNIP Protocol

The properties documented here remain **in force** and form
the foundation of the new SNIP protocol:

- BobClaim remains the **primary** spending path for
  Receipt outputs.
- The AgSig binding remains the basis for ensuring that only
  Bob can claim.
- The hash binding to `SharedReceiptKeyHashes.bob` remains
  the basis for Alice and outsiders to verify attribution.

**Change for the SNIP protocol:**
- The Alice-reclaim path uses the **ordinary `Plain` kernel**
  from Epic Cash (PROTOCOL.md §3.9). There is no separate
  `AliceReclaim` kernel type.
- A consensus adjustment is required to allow `Plain` kernels
  for `Receipt` inputs (see CURRENT_STEP.md, options A and B).
- First-Confirmed-Spend-Wins: if Bob has already claimed, a
  later Alice spend is rejected (`AlreadySpent`). The reverse
  holds as well.

**New for the SNIP protocol:**
- Bob must recognize payments **by scanning the chain**
  (not via wallet message).
- Bob must **detect and claim three independent payments**
  in parallel.
- The final outputs must be **exclusively spendable by
  Bob**.
- A `PaymentDescriptor` abstraction must exist.
- A new integration test must demonstrate the 3-payment
  flow end-to-end.
- Alice can verify from the chain that Bob has taken over
  the payments (PROTOCOL.md §3.8).
- Alice can claim the shared outputs herself at any time,
  as long as Bob has not already transferred them
  (PROTOCOL.md §3.9).

These new points are in
[CURRENT_STEP.md](../CURRENT_STEP.md) and
[PROTOCOL.md](../PROTOCOL.md).