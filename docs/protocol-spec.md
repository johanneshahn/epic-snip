# Protocol Specification — superseded by PROTOCOL.md

**Status: HISTORICAL and superseded by
[PROTOCOL.md](../PROTOCOL.md).**

This document described the early in-memory POC modeling
in `src/one_sided.rs`. It is **not** the currently valid
protocol description. The top priority and the
complete functional specification are in
[PROTOCOL.md](../PROTOCOL.md).

---

## Historical Content

### Model (In-Memory, no chain)

`src/one_sided.rs` modeled a one-sided
shared-redeemable payment flow. Bob generated a
long-term `ReceiverKey` (secp256k1) and published
the public scan key. Alice generated a fresh
ephemeral key `a` for each payment and derived a
domain-separated shared blind from
`ECDH(a, B)`. From this blind a Pedersen commitment
and a `PendingEnvelope` with the ephemeral public key
`A` were created.

Bob received the envelope through a modeled one-sided
wallet message channel, scanned it with `b`, computed
`ECDH(b, A)` and recognized his payment only on a
matching commitment. He then received a
`ClaimCapability` with the shared blind.

### State Machine (In-Memory)

The `OneSidedLedger` had three states:

- `Pending`: output not yet spent.
- `Claimed`: Bob transferred the output to a key
  exclusively controlled by him.
- `SenderSpent`: Alice spent the output competitively.

Terminal states have no outgoing transitions. The ledger
accepted only the first valid path: `Claim XOR
SenderSpend`.

### AgSig Authorization

The AgSig signature bound `payment_id`, path
(`claim`/`sender-spend`), origin commitment,
destination commitment, and amount. A claim
authorization could not be repurposed for
`sender_spend` and vice versa.

## Why Superseded

This in-memory specification fulfilled **none** of the
binding requirements defined in
PROTOCOL.md §3:

| Requirement (PROTOCOL.md §3) | POC fulfillment |
| --- | --- |
| 3.5 Epic Cash consensus | ❌ no consensus |
| 3.2 Bob determines amount from the chain | ⚠️ only via wallet message |
| 3.1 Bob recognizes without communication | ⚠️ via wallet message, not chain scan |
| 3.7 No superfluous linkable info | ⚠️ wallet message contains payment detail |

The currently valid specification requires the
complete demonstration in the Usernet chain
without external wallet messages.