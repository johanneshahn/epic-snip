# Assumptions and Open Questions — superseded by PROTOCOL.md

**Status: HISTORICAL and superseded by
[PROTOCOL.md](../PROTOCOL.md).**

This document described the assumptions and open questions of the
in-memory POC. It is **not** the current list for the SNIP
implementation.

---

## Historical Assumptions

- secp256k1, Pedersen commitments, AgSig and SHA-256
  were only used as established library primitives;
  the protocol construction itself was not proven.
- The amount was visible in the model; the Pedersen
  commitment provided no amount confidentiality in the
  isolated POC.
- A `payment_id` bound commitment,
  ephemeral key, and receiver key, but replaced neither
  consensus serialization nor output binding.
- The ECDH blind derivation was a heuristic
  construction; a formal security analysis was pending.
- Key derivation, descriptor reuse protection, backup,
  recovery, reorg, and mempool rules were missing.
- The reliability of the external
  `PendingEnvelope` channel was not modeled:
  lost or tampered envelopes were not detected.

Before production use, formal specification and proofs,
a consensus-capable design, integration of real confidential
transactions, and independent cryptography and
security audits would be required. This project
claims neither Epic Cash consensus compatibility nor
real protocol security.

## Open Questions for the New SNIP Protocol

The target implementation defined in PROTOCOL.md
brings the following open questions:

1. **Payment descriptor format.** What data must the
   descriptor contain so that Alice can derive
   arbitrarily many payments from it without involving
   Bob? (Proposal: Bob's long-term scan public key
   + receipt-spend public key + version and
   domain separator.)
2. **Stealth controllability.** Alice chooses a fresh
   ephemeral key per payment. How is the corresponding
   stealth derivation canonically defined so that Bob
   can deterministically reproduce it without
   communication?
3. **Scan efficiency.** How does Bob's wallet
   scan the chain efficiently without checking every
   `Output` against every possible
   `descriptor_id`? (Proposal: index over
   `H(descriptor_id)` in
   `SharedReceiptKeyHashes`.)
4. **Multi-payment correlatability.** How is it
   prevented that three payments from the same
   descriptor become publicly linkable (PROTOCOL.md §3.7)?
5. **Amount leakage during scan.** Does the scan
   additionally reveal amount information beyond the
   output itself, beyond the range proof?
6. **Descriptor rotation.** How is the descriptor
   rotated when Bob's key is compromised, without
   discarding ongoing payments?
7. **Reorg handling.** What happens to Bob's
   scan results when a block is reorged? (Proposal:
   scan only on confirmed blocks.)
8. **Claim order.** Can Bob bundle multiple claims in
   one transaction, or must each claim be its own
   transaction? (PROTOCOL.md §2.5 suggests
   "own transaction".)
9. **End-output exclusivity.** How is the exclusivity
   of the final outputs for Bob cryptographically
   proven (PROTOCOL.md §3.4)?
10. **Wallet integration.** How are descriptor
    generation, scan, and claim integrated into the
    existing Epic wallet without endangering
    Mainnet compatibility?

These questions are addressed in the next design pass.
Until then, PROTOCOL.md remains the top priority.