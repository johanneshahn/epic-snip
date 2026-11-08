# Test Plan — superseded by PROTOCOL.md

**Status: HISTORICAL and superseded by
[PROTOCOL.md](../PROTOCOL.md).**

This test plan described the POC tests in `tests/` and
does **not** cover the 3-payment demonstration
required today.

## New Top-Priority Tests (from PROTOCOL.md §5)

The acceptance integration test to be written
(`upstream/epic/chain/tests/snip_acceptance.rs`) must
demonstrate **a single** flow:

1. Bob generates a Payment Descriptor (once).
2. Alice creates 3 independent payments from the
   descriptor — without Bob, without communication.
3. All 3 payments are confirmed in the local Usernet chain.
4. Bob scans the chain (without communication with Alice),
   detects all 3 payments, and determines the amounts.
5. Bob claims each of the 3 payments in a separate
   transaction.
6. All 3 claims are confirmed in the Usernet chain.
7. The final outputs are exclusively spendable by Bob
   (test: any other attempt to spend them is rejected).

```sh
cargo test -p epic_chain --test snip_acceptance
```

## Existing, Completed Tests (historical)

| Test | Coverage | Status |
| --- | --- | --- |
| `tests/one_sided_pending.rs` | In-memory POC: scan, claim, sender-spend, AgSig binding | ✅ completed |
| `upstream/epic/chain/tests/receipt_claim_roundtrip.rs` | 1 payment: funding → claim → confirmation in Usernet chain | ✅ completed |
| `upstream/epic/chain/tests/spend_attribution.rs` | BobClaim unforgability, attribution, no Alice-spend path | ✅ completed |
| `cargo test -p epic_core --lib` | Core crypto, serialization, transaction validation | ✅ completed |

These tests are **historical** and are no longer
modified. They serve research and as the foundation
for the new 3-payment flow.

## What the POC Test Plan Covered (historical)

The integration tests verified:

- Bob recognizes and claims a `OneSidedPayment` via his
  scan key.
- A different receiver key does not recognize the envelope.
- Alice can spend the output competitively before Bob
  claims; a later Bob claim is then rejected
  (`AlreadySpent`).
- The AgSig authorization is bound to `payment_id` and
  commitment.
- Pedersen commitments have the expected additive
  property.

For every invalid authorization an error without state
change was expected, and for competing valid attempts
exactly one successful transition. Not covered were real
networks, consensus, reorgs, side channels, randomness
quality, persistence, load, DoS, formal verification,
and real Mimblewimble cryptography. Successful tests
were not a security proof.