# Cryptographic Foundation — superseded by PROTOCOL.md

**Status: HISTORICAL and superseded by
[PROTOCOL.md](../PROTOCOL.md).**

This document described the cryptographic foundations of the
in-memory POC. It is **not** the cryptography specification of
the target protocol defined in PROTOCOL.md.

---

## Historical Content

This module used the local Git submodules `upstream/epic` and
`upstream/epic-wallet` as reference. Both pinned the same
`EpicCash/rust-secp256k1-zkp` revision
`1131a20920576413cd949e012a160561080324c1` in their
`Cargo.lock` files; this revision is also explicitly pinned here.
Only the tested interfaces
`Secp256k1::commit`, `commit_sum`,
`verify_commit_sum`, `ecdh::SharedSecret` and
`aggsig::{sign_single, verify_single}` were used.

`src/crypto.rs` encapsulated the Pedersen and
AgSig primitives. `src/one_sided.rs`
additionally used `ECDH` via
`SharedSecret` to derive a shared blind. The blind was hashed
in a domain-separated way with domain separator, ephemeral key,
receiver key, amount, and counter value until a valid
secp256k1 scalar was produced.

The AgSig authorization bound `payment_id`,
path (`claim`/`sender-spend`), origin
commitment, destination commitment, and amount. Thus a
claim authorization could not be verified for
`sender_spend` or with a modified context.

There were deliberately no blocks, no
UTXO database, no network communication,
no wallet key management, and no
real consensus objects.

## Relation to the new SNIP specification

The primitives documented here are still the
foundation of the cryptography implemented in
`upstream/epic`:

| POC primitive | SNIP equivalent in `upstream/epic` |
| --- | --- |
| `ReceiverKey` (secp256k1) | `receipt_spend_secret_key` + `scan_public_key` in Bob's descriptor |
| `ECDH(a, B)` → shared blind | `ECDH(r_i, S)` → `h_i` + `q_i` (Receipt-Claim design) |
| Pedersen commitment | Output commitment + range proof |
| AgSig (single) | `ReceiptClaim` signature over `(path, original_commitment, kernel_excess, fee)` |
| `PendingEnvelope` (wallet message) | **No longer used** — Bob scans the chain directly |

The concrete stealth derivation and the
`BobClaim` consensus rules are in
[receipt-claim-design.md](receipt-claim-design.md)
and in `upstream/epic/core/src/receipt_claim.rs`.