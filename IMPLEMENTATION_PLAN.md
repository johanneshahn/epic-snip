# Implementation Plan — superseded by PROTOCOL.md

**Status: HISTORICAL.** This plan described the
in-memory POC phase of the project. It is **completed and
superseded by the functional specification in
[PROTOCOL.md](PROTOCOL.md)**.

The top priority of this project today is the
complete demonstration of the 3-payment flow
(descriptor → 3 independent payments → 3 confirmations →
Bob scans → Bob claims → final outputs spendable only by
Bob), not the in-memory ledger simulation documented here.

---

# Historical Plan: In-Memory POC

## Goal

Create a reproducible cryptographic research prototype in the
existing project folder for a hypothetical Epic Cash SNIP
protocol:

- Alice creates a payment without Bob being online.
- Bob can claim the payment at any time.
- Alice can execute a refund only from a defined block height.
- Exactly one of the two paths can succeed.

Important: this is exclusively a local modeling and test
environment. It is not an Epic Cash wallet, not a blockchain
implementation, not consensus-compatible, and must not use
real keys, funds, or networks.

## Technical Decisions

- Language: Rust
- Build/Test: Cargo
- Model: deterministic in-memory ledger simulation
- Cryptography: key and signature wrappers from an established
  Rust library; no self-developed cryptography
- Time: exclusively simulated block heights, no system time
- Persistence: none; test data only in memory
- Network: none

## Directory

| Path | Content |
| --- | --- |
| `src/one_sided.rs` | Core of the POC modeling |
| `tests/` | POC integration tests |
| `upstream/epic` | Local fork for the chain phase (completed) |
| `docs/` | Research documentation |

## Phases (all completed)

1. Model design and state machine.
2. Crypto building blocks (ECDH, Pedersen, AgSig).
3. `OneSidedLedger` with claim/refund decision.
4. POC integration tests.
5. Handover to the chain phase (`upstream/epic`).
6. AliceReclaim removal and spend-attribution research.
7. **[New, ACTIVE]** Complete 3-payment demonstration
   per [PROTOCOL.md](PROTOCOL.md).