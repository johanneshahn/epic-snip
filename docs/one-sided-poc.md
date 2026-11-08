# One-Sided Shared-Redeemable POC — HISTORICAL

**Status: COMPLETED and superseded by
[PROTOCOL.md](../PROTOCOL.md).**

This document describes the in-memory modeling implemented in
`src/one_sided.rs`. It is **not** the target implementation
required by PROTOCOL.md and not guidance for further
development. The complete 3-payment flow (descriptor → 3
independent payments → 3 confirmations → Bob scans → Bob claims →
final outputs spendable only by Bob) is realized in
`upstream/epic`; this in-memory model plays no role in
acceptance.

---

## Purpose and Scope (historical)

`src/one_sided.rs` modeled a flow without a response from
Bob during payment creation. It deliberately followed **no**
on-chain "pending" status and no refund path. The original output
was automatically recognizable by Bob and jointly spendable by
Alice and Bob:

1. Bob generated `ReceiverKey` and published only
   its public scan key.
2. Alice generated a fresh ephemeral key `a` for **each**
   payment and derived a domain-separated shared blind from
   `ECDH(a, B)`.
3. From this blind a Pedersen commitment and a
   `PendingEnvelope` with the ephemeral public key `A`
   were created.
4. Bob received the envelope through a modeled
   one-sided wallet message channel, scanned it with `b`,
   computed `ECDH(b, A)` and recognized his payment only on
   a matching commitment.
5. Bob **had to** transfer the recognized output to a new
   commitment with a key known only to him. Only after
   confirmation of this transfer was the semi-interactive
   transfer complete.

The designation `PendingEnvelope` in the code named only the
external wallet message. It must not be understood as a status
enforceable by Epic consensus.

## What the Code Actually Tested (historical)

- ECDH produced the same derived blind on Alice's and Bob's
  side.
- A different scan key did not recognize the envelope.
- Bob's mandatory final transfer moved the local ledger from
  `Pending` to `Claimed`.
- An earlier Alice transition set `SenderSpent`; a later Bob
  spend was rejected afterward. This test documented the
  consciously accepted First-Spend-Wins semantics.
- The AgSig authorization bound payment ID, origin and
  destination commitment, amount, and path to each other.

## Explicit Non-Properties (historical)

This was neither a security proof nor an Epic Cash protocol.
In particular:

- `PendingEnvelope` was an external wallet message. The
  current Epic output and the existing range-proof message
  did not provably carry the required 33-byte ephemeral key.
- The `OneSidedLedger` built no transaction, no kernel, and
  no range proof. It enforced neither a Mimblewimble balance
  equation nor UTXO, mempool, or reorg rules.
- The target key was only represented in the model as Bob's
  own output commitment.
- Alice could also derive the shared blind and therefore
  spend competitively before Bob's mandatory final transfer.

## Relation to the New SNIP Specification

The modeling documented here **did not include** a
Usernet chain phase, a real consensus path, a
multi-payment flow, or Bob-only final outputs. It served
exclusively for early research into cryptographic
foundations. The currently valid objective is in
[PROTOCOL.md](../PROTOCOL.md).