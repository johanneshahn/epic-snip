# Threat Model — superseded by PROTOCOL.md

**Status: HISTORICAL and superseded by
[PROTOCOL.md](../PROTOCOL.md).**

This document described the threat model of the
in-memory POC. It is **not** the threat model of the
target protocol defined in PROTOCOL.md.

---

## Historical Content

Attackers could manipulate `PendingEnvelope` messages,
adjust payment IDs, or spend simultaneously from Alice
and Bob. The in-memory ledger rejected manipulated
or path-bound AgSig signatures. The central state
change prevented a second spend within the same ledger.

Assumed were the unforgability of
secp256k1-AgSig, protection of private keys
(Alice's ephemeral key, Bob's receiver key)
and atomic execution of a
`OneSidedLedger` call. Key loss or theft,
DoS, censorship, data races without suitable
external synchronization, serialization errors,
hash collisions, and memory exhaustion were not
solved. With parallel use the caller had to
protect the ledger, for example with a `Mutex`.

Not modeled were network, mempool, reorgs,
fees, UTXO consensus, real wallets,
reliability of the `PendingEnvelope` channel, and
amount confidentiality. This was not a formal
security analysis and made no statement about
Epic Cash or Mimblewimble.

## Open Threat Questions for the New SNIP Protocol

The target implementation defined in PROTOCOL.md
brings new threat questions that still need to be
analyzed and documented:

- **Descriptor leakage:** Can an attacker infer Bob's
  long-term key material from the payment descriptor?
- **Multi-payment correlation:** Are the three
  independent payments publicly linkable
  (PROTOCOL.md §3.7)?
- **Scan correlatability:** Can an observer
  correlate Bob's scan activity with
  Alice's payments?
- **Claim correlatability:** Can Bob's claim
  transaction be linked to a specific payment
  output?
- **Descriptor reuse:** How many payments can Alice
  derive from a descriptor before a reuse
  signature becomes noticeable?
- **Amount leakage:** Does the scan reveal
  additional amount information beyond the output
  itself, beyond the range proof?
- **Descriptor rotation:** How is the descriptor
  rotated when Bob's key is compromised, without
  discarding ongoing payments?

These questions are addressed in the research phase
after the complete 3-payment demonstration. They are
**not** part of the current acceptance.