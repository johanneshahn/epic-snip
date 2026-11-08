# Receipt-Claim & SNIP: One-Sided Reusable Payments

Status: experimental, local Usernet implementation.
Neither consensus-compatible nor cryptographically proven;
not suitable for Mainnet, Floonet, or real funds.

> **Top priority.** All content of this document serves the
> transaction flow defined in [PROTOCOL.md](../PROTOCOL.md) §2
> and §5:
>
> 1. Bob publishes a **Payment Descriptor** once.
> 2. Alice creates **three independent payments** from it
>    (without Bob, without communication).
> 3. All three payments are confirmed in the **local
>    Usernet chain**.
> 4. Bob **detects** all three **by scanning the chain**
>    (without communication with Alice) and **determines
>    amounts**.
> 5. Bob **claims** each of the three payments in a
>    **separate transaction**.
> 6. All three claims are confirmed in the Usernet chain.
> 7. The **final outputs** are **exclusively spendable by
>    Bob**.
>
> Per [PROTOCOL.md §6](../PROTOCOL.md): a successful
> isolated crypto or consensus test **does not replace** the
> complete 3-payment demonstration.

## Goal

The shared output created by Alice may only be spent via a
verifiable resolution path. Bob must transfer it to a key
exclusively controlled by him, so that the transfer is
complete. Bob must not be able to perform this transfer
without simultaneously leaving a claim verifiable by Alice.

**Alice has a competing reclaim path.** As long as Bob has not
transferred the output into his own outputs, Alice can claim
it herself. The first confirmed valid spend wins
(First-Confirmed-Spend-Wins). The Alice-reclaim path uses the
existing standard mechanism of Epic Cash (ordinary `Plain`
kernel) and does not need to be reinvented
(PROTOCOL.md §3.9).

**Alice can verify the claim.** From the chain, Alice can
follow that Bob has taken over the payments (PROTOCOL.md §3.8).

At the same time, the chain must not reveal which later
destination output belongs to Bob. An outsider should only be
able to recognize that a special output was claimed; not which
new output is Bob's final one.

**New compared to the earlier POC specification:**
Bob must recognize payments **from the chain** — not from an
external wallet message. He must **determine amounts** from
the chain. He must **detect and claim three independent
payments** in parallel. The final outputs must be
**exclusively spendable by Bob**. **Alice can verify the
claim** and has a **competing reclaim path**
(PROTOCOL.md §3.8 and §3.9).

## Roles and One-Time Setup

Bob generates long-term key material once and publishes the
public **Payment Descriptor**. The descriptor is reusable:
from it Alice can derive arbitrarily many payments without
involving Bob again.

The descriptor contains:

```text
PaymentDescriptor:
  version        : u8
  descriptor_id  : [u8; 32]   // Domain-Separator + version hash
  scan_public    : PublicKey  // sG, s = scan_secret
  receipt_spend_public : PublicKey  // pG, p = receipt_spend_secret
```

Bob's private keys (`scan_secret`, `receipt_spend_secret`)
remain exclusively in Bob's wallet. Alice stores the
descriptor; she needs no new response from Bob per payment.

**Caution:** The descriptor is **public**. Any party can derive
payments from it. Only Bob can **recognize** payments (via
`scan_secret`) and **claim** them (via
`receipt_spend_secret`).

## Stealth-Receipt Derivation

The local research implementation uses two domain-separated
ECDH-based derivations. It is not a cryptographic proof and
not a final protocol specification; the following notation
describes the implemented research approach:

```text
Bob-Descriptor: ScanPublic S = sG, ReceiptSpendPublic P = pG
Alice per payment i:
  r_i random, Hint R_i = r_i G
  shared value: h_i = H(domain || ECDH(r_i, S) || R_i || descriptor_id)
  Bob-Receipt-Public-Key: B_i = P + h_i * G
  Bob-Receipt-Private-Key: b_i = p + h_i   (only Bob knows)

Shared-Spend-Blind:
  q_i = HashToScalar(
    separate_domain || ECDH(r_i, S) || R_i || S
  )
```

Bob scans `R_i`, computes `ECDH(s, R_i)` and thereby `b_i`;
Alice can compute `B_i` but not `b_i`. The same ECDH output
lets both sides derive `q_i`: Alice knows `r_i`, Bob knows
`s`. From this both can form the same commitment blind for the
shared special output. The derivation for `q_i` is explicitly
separated from that for `h_i`, so that a receipt authorization
key is not reused as a spend blind.

`R_i` is present in the local implementation as a canonically
serialized 33-byte stealth hint on the Receipt output. The core
builds, in the local research API, a Receipt output with `q_i`,
rewinds its amount for Bob, and creates a fully
`Transaction::validate(...)`-verifiable BobClaim transaction
from it.

## Consensus Format (Usernet-only)

A Receipt output binds at creation time:

```text
SharedReceiptKeyHashes:
  bob   : [u8; 32]   // H(domain || B_i)
  alice : [u8; 32]   // H(domain || A_i)  // for Alice-reclaim
  hint  : [u8; 33]   // R_i (canonically serialized)
```

**Important:** The `alice` hash is required for the
Alice-reclaim path. Both hashes are fixed at output creation
time and are part of the UTXO identity.

A `BobClaim` kernel contains:

```text
BobClaim:
  original_commitment = C
  receipt_public_key = B_i
  receipt_signature = Sign_{b_i}(
    domain || path || C || kernel_excess || fee
  )
```

The Alice-reclaim path uses the **ordinary `Plain` kernel**
from Epic Cash. There is no separate `AliceReclaim` kernel
type. Alice builds a standard Epic transaction that spends
the `Receipt`-feature output.

Node validation checks:

1. `C` is a Receipt output spent as input of this
   transaction.
2. For `BobClaim`: the stored key hash matches the Bob key
   (`hashes.bob == H(B_i)`) and the `receipt_signature`
   over the canonically defined binding value is valid.
3. All normal rules for transaction balance, range proofs,
   fee, and kernel signature are also satisfied.
4. Each spend of a Receipt output contains exactly one
   matching resolution kernel (either `BobClaim` or
   `Plain`).
   *(Note: the current consensus rule requires BobClaim.
   An adjustment is required to satisfy §3.9 — see
   CURRENT_STEP.md.)*
5. **First-Confirmed-Spend-Wins:** If Bob has already
   claimed (output is consumed), a later Alice spend is
   rejected (`AlreadySpent`). The reverse holds as well.

## Scan for Bob (new part still to be implemented)

**This API is not yet implemented in
`upstream/epic`.** It is the most important building block
missing for SNIP acceptance.

```rust
// Proposed scan API
pub fn scan_for_bob(
    receipt_spend_secret_key: SecretKey,
    scan_secret_key: SecretKey,
    descriptor: &PaymentDescriptor,
    chain: &ChainHandle,
) -> Vec<DetectedPayment>
{
    // For each Receipt output in the chain:
    //   1. ECDH(scan_secret, hint) → h_i
    //   2. B_i = P + h_i * G
    //   3. H(B_i) == hashes.bob? → detection!
    //   4. derive q_i → rewind amount
    // → DetectedPayment { output_index, amount, hint }
}
```

Each `DetectedPayment` contains enough information for Bob to
build his claim transaction without contacting Alice.

**Efficiency question (open):** The naive scan checks every
`Receipt` output against every `descriptor_id`. For Usernet
acceptance this suffices; for a production wallet an index
over `H(descriptor_id)` in `SharedReceiptKeyHashes` would be
required.

## Proposed 3-Payment Flow (Acceptance)

```text
Bob:
  d = PaymentDescriptor.create()
  // publishes d to Alice (once)

Alice:
  payment_1 = Payment.create(d, amount=100, alice_blind_1)
  payment_2 = Payment.create(d, amount=200, alice_blind_2)
  payment_3 = Payment.create(d, amount=300, alice_blind_3)
  // brings all three into Usernet blocks:
  mine_block(payment_1)
  mine_block(payment_2)
  mine_block(payment_3)

Bob (without communication with Alice):
  detected = scan_for_bob(
    receipt_spend_secret, scan_secret, d, chain
  )
  // detected.length == 3
  // detected[0].amount == 100
  // detected[1].amount == 200
  // detected[2].amount == 300

  for payment in detected:
    claim_tx = Claim.build(
      receipt_spend_secret, scan_secret, d, payment
    )
    mine_block(claim_tx)
  // all three claims confirmed

Alice (verification, PROTOCOL.md §3.8):
  for payment in [payment_1, payment_2, payment_3]:
    verify_bob_claimed(payment)
    // Alice can follow from the chain
    // that Bob has claimed the output and that
    // the funds now reside in Bob's final outputs.

Final outputs:
  // each final output is spendable only by Bob's
  // target key; any other spend attempt is rejected.

Alternative: Alice-Reclaim (PROTOCOL.md §3.9):
  // If Bob does NOT claim:
  // Alice builds an ordinary Epic transaction
  // (Plain kernel) that spends the shared output.
  alice_spend = PlainSpend.build(
    shared_blind, destination_blind
  )
  mine_block(alice_spend)
  // Alice spend succeeds, payment goes
  // to Alice's own output.
  // Bob can no longer claim the output
  // (already spent).
```

## Local Implementation State

In the local subproject `upstream/epic`, a first
Usernet-only core step is implemented and covered by
core unit tests:

- `OutputFeatures::Receipt` carries a Bob-only
  `SharedReceiptKeyHashes` (`bob: [u8; 32]`) plus a
  canonically validated 33-byte stealth hint. Input, output,
  and MMR output identifier serialize these data.
- The core contains an experimental ECDH/tweak primitive:
  Alice can form a Bob receipt public key from Bob's
  public descriptor and a fresh ephemeral key, while Bob
  derives the corresponding private receipt key from the
  hint and his private descriptor keys.
- The core also derives a separate shared spend blind.
  Alice and Bob independently obtain the same
  commitment for the special output. The research builder
  creates a valid Receipt range proof from it; Bob can
  read its amount with a rewind nonce derived from the
  shared blind.
- A local roundtrip then builds a one-input/one-output
  BobClaim transaction with a normal destination output,
  valid range proof, kernel signature, receipt signature,
  and correct kernel sum. The test calls
  `Transaction::validate(...)` in Usernet mode.
- `libtx::build::receipt_output(...)` creates an
  experimental Receipt output with key hashes and hint;
  the regular proof builder still creates its range proof.
- `KernelFeatures::BobClaim` (U8 = 3) carries the
  respective fee and a `ReceiptClaim` transcript.
- The transcript binds, via a domain-separated signature,
  path, original commitment, kernel excess, and fee. The
  node first checks the normal kernel signature and then
  this binding.
- A transaction-body check requires a one-to-one
  mapping between each spent Receipt input and a matching
  resolution kernel plus registered receipt-key hash.
- Receipt features are rejected outside Usernet mode
  in this experimental stage.
- The Alice-reclaim path uses the ordinary `Plain` kernel
  from Epic Cash (PROTOCOL.md §3.9). There is no
  separate `AliceReclaim` kernel type.

A local chain integration test
(`receipt_claim_roundtrip.rs`) creates a Usernet genesis
block, mints a mature test coinbase, includes Alice's
receipt funding transaction in a block, and includes Bob's
claim in a later block. It then verifies that the receipt
UTXO is consumed, Bob's destination output is unspent, and
full TxHashset validation succeeds.

**Not yet available:**

- The `scan_for_bob` API (see above)
- An integration test demonstrating three independent
  payments in one run
- The `PaymentDescriptor` abstraction as its own type
  (currently implicit via `receipt_spend_public_key` +
  `scan_public_key`)
- Wallet integration of descriptor generation, scan, and
  claim
- P2P versioning, mempool admission, running
  node/miner integration

## What Is Public and What Stays Private

| Information | Visibility |
| --- | --- |
| Payment Descriptor | permanently public (published by Bob) |
| Stealth hint `R_i` per payment | public, but fresh per payment; without Bob's scan key no public link to `B_i` |
| Special output `C` was claimed | permanently public via the Receipt-Claim kernel |
| A one-time receipt key resolved it | permanently public |
| Bob's final destination output | not part of the Receipt-Claim kernel; only Bob recognizes it during wallet scan |
| Amount | still hidden by commitment and range proof, unless a party reveals it |
| Bob's long-term key material | **never** public |

## Atomic Property

A Bob claim is only valid if its receipt signature is
contained in a valid `BobClaim` kernel. The Alice reclaim
uses the ordinary `Plain` kernel and needs no additional
receipt signature.

**First-Confirmed-Spend-Wins.** If Bob has already claimed
(output is consumed), a later Alice spend is rejected
(`AlreadySpent`). The reverse holds as well: if Alice has
spent the output, Bob can no longer claim it.

**Alice can verify the claim.** From the chain, Alice can
follow that Bob has claimed the output by finding the
`BobClaim` kernel and checking the signatures and key
hashes (PROTOCOL.md §3.8).

## Open Questions

1. **Scan efficiency.** How does Bob's wallet scan scale?
   (For Usernet acceptance the naive scan suffices.)
2. **Descriptor index.** Should `SharedReceiptKeyHashes`
   contain a separate index over `H(descriptor_id)` so the
   scan does not check every `Receipt` output against
   every `descriptor_id`?
3. **Multi-payment correlatability.** Can three payments
   from the same descriptor be publicly correlated?
   (PROTOCOL.md §3.7.)
4. **Amount leakage during scan.** Does the scan reveal
   additional amount information beyond the output itself,
   beyond the range proof?
5. **Claim order.** Can Bob bundle multiple claims in
   one transaction, or must each claim be its own
   transaction? (PROTOCOL.md §2.5 suggests "own
   transaction".)
6. **End-output exclusivity.** How is the exclusivity of
   the final outputs for Bob cryptographically proven
   (PROTOCOL.md §3.4)?
7. **Reorg handling.** What happens to Bob's scan results
   when a block is reorged?
8. **Wallet integration.** How are descriptor generation,
   scan, and claim integrated into the existing Epic
   wallet without endangering Mainnet compatibility?
9. **Alice-reclaim consensus adjustment.** How is
   consensus adjusted so that a `Plain` kernel may spend a
   `Receipt` input while Bob claims still require
   `ReceiptClaim` attribution? (See CURRENT_STEP.md,
   options A and B.)
10. **Alice verification.** How exactly can Alice verify
    from the chain that Bob has taken over the payments?
    (PROTOCOL.md §3.8.)
11. **Race condition.** What happens when Alice and Bob
    simultaneously attempt claim/reclaim? How is the
    winner determined?

These questions are addressed in the next design pass.
Until then, PROTOCOL.md remains the top priority.