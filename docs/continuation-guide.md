# Handover: Further Work on Epic SNIP — superseded

**Status: HISTORICAL and superseded by
[PROTOCOL.md](../PROTOCOL.md).**

This document described the handover point from the
in-memory POC phase to the chain phase. It is **no longer**
valid. Today's active work is tracked in
[CURRENT_STEP.md](../CURRENT_STEP.md).

---

## Historical Content

### Starting point (historical)

The POC modeled a one-sided
shared-redeemable payment flow. Alice created an
output without a response from Bob, which Bob could
recognize through a published scan key and an external
wallet message. Both Alice and Bob
derived a shared spending capability. Bob
had to transfer the output to a key exclusively
controlled by him. Only its
confirmation completed the semi-interactive transfer.

The code under `src/one_sided.rs` and the tests under
`tests/` verified these properties only in
memory. They modeled neither a UTXO set
nor a real Mimblewimble transaction.

There was deliberately no consensus status "pending" and no
requirement to lock Alice out before this mandatory
final transfer.

### Further work (historical)

The further work in `upstream/epic` included:

- Usernet chain roundtrip (1 payment)
- BobClaim consensus path
- Spend-attribution research
- AliceReclaim removal
- Verdict **VERIFIED**

All of these points are **completed** and documented in their
respective documents.

## Transition to the new SNIP specification

Today's task is the complete demonstration of the
3-payment flow per
[PROTOCOL.md](../PROTOCOL.md) §5. The exact status,
open questions, and next steps are in
[CURRENT_STEP.md](../CURRENT_STEP.md).