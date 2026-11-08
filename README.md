# Epic SNIP

**Semi-Non-Interactive Payments for Epic Cash — Proof of Concept**

Epic SNIP (Semi-Non-Interactive Payments) is an experimental research
implementation that investigates whether Epic Cash can support payments where
the sender creates a transaction while the receiver is offline, without
abandoning Epic's existing Mimblewimble cryptographic foundations.

This is a **research prototype**. It is not intended for production deployment,
has not been cryptographically audited, and is not compatible with Epic
Mainnet or Floonet. All consensus changes are Usernet-only.

## The Problem

Ordinary Mimblewimble transactions are interactive: sender and receiver must
exchange transaction data (blinding factors, range proofs, kernels) before the
transaction can be finalized and broadcast. This requires both parties to be
online or to use an out-of-band communication channel.

The research question Epic SNIP addresses:

> Can Epic Cash support payments where the sender creates a transaction while
> the receiver is offline, without abandoning the existing Mimblewimble
> cryptographic foundations?

## The Approach

Bob publishes a **reusable payment descriptor** once. Alice uses that descriptor
to create payments independently, without Bob being online or involved.

Each payment creates a special **Receipt** output on the chain. Bob can later
scan the chain, discover Receipts addressed to him, and **claim** them into
ordinary Epic outputs. Alice can **reclaim** an unspent Receipt without Bob's
cooperation.

Both parties hold independent authorization keys. The first confirmed valid
spend consumes the Receipt UTXO and prevents the competing spend. Until Bob
successfully claims a Receipt, the payment is not final — Alice can still
reclaim it.

## How It Works

```mermaid
sequenceDiagram
    participant Alice
    participant Chain as Epic Chain
    participant Bob

    Bob->>Alice: Publish reusable descriptor
    Note over Bob: Bob can go offline
    Alice->>Chain: Create and broadcast Receipt
    Chain-->>Chain: Confirm Receipt
    Note over Bob: Bob returns later
    Bob->>Chain: Scan for Receipts
    Chain-->>Bob: Matching Receipt discovered
    Bob->>Chain: Submit BobClaim
    Chain-->>Bob: Ordinary Epic output confirmed
```

The two alternative spending paths compete for the same UTXO:

```mermaid
flowchart TD
    R["Unspent Receipt UTXO"]
    B["BobClaim — Bob receives payment"]
    A["AliceSpend — Alice reclaims payment"]
    S["Receipt UTXO spent"]

    R --> B
    R --> A
    B --> S
    A --> S
```

Only one path can succeed. The first confirmed valid spend wins.

## Cryptographic Design — High-Level Overview

- **Shared payment blind `q_i`**: both Alice and Bob can derive this from
  Alice's ephemeral secret and Bob's descriptor. It is a balance-construction
  capability, not an authorization.
- **Independent Alice authorization key `a_i`**: derived from Alice's seed.
  Only Alice can produce a valid AliceSpend authorization.
- **Independent Bob authorization key `b_i`**: derived from Bob's descriptor
  secrets. Only Bob can produce a valid BobClaim.
- **Receipt commitment**: a Pedersen commitment hiding the value, with a
  Bulletproof range proof.
- **Authorization verification**: consensus checks that the revealed key hashes
  to the committed value and that the signature verifies.

Knowing the shared blind does not by itself authorize either spending path.
The separation comes from independent authorization keys, not from the blind.

See [PROTOCOL.md](PROTOCOL.md) and
[docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md](docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md)
for implementation details.

## What Was Implemented

- **Cryptographic foundations**: shared receipt keys, Alice authorization,
  Bob receipt key, domain-separated derivation.
- **AliceSpend authorization**: Plain kernel with input-level authorization.
- **BobClaim authorization**: existing ReceiptClaim mechanism, unchanged.
- **Receipt-aware consensus validation**: two-branch resolution, signature
  verification, cut-through exemption.
- **Offline payment creation**: Alice builds funding transactions without Bob.
- **Receipt scanning and discovery**: Bob discovers payments by scanning.
- **Alice reclaim**: Alice reclaims unspent Receipts independently.
- **Bob claim**: Bob claims discovered Receipts into ordinary outputs.
- **Reusable `epic_snip_wallet` library**: deterministic derivation,
  persistence, recovery, single-writer lock.
- **Chain integration**: adapter bridging the wallet to `epic_chain`.
- **Security and architecture review**: threat-model review (Step 3F),
  wallet hardening (Step 3G).

## Proof of Concept Results

The following scenarios were demonstrated and verified:

| Capability | Verification |
|---|---|
| Three independent payments to one descriptor | `phase_a` |
| Bob offline during creation | `phase_a` |
| Bob discovers all three by scanning | `phase_b` |
| Bob claims R1 | `phase_c` |
| Alice reclaims R2 | `phase_c` |
| Bob claims R3 after wallet restart | `phase_c` |
| Double-spend rejected | `phase_d`, `double_spend_rejection` |
| Bob-claim-wins order | `phase_e1` |
| Alice-reclaim-wins order | `phase_e2` |
| Seed recovery (Bob, Alice) | `snip_wallet_recovery` (15 tests) |
| Key separation and domain separation | wallet unit tests (10 tests) |

All transactions were validated through the actual Epic chain implementation,
not merely simulated. See [POC_RESULTS.md](POC_RESULTS.md) for the full
test-to-capability mapping.

## Running the Proof of Concept

From the `upstream/epic` directory:

```sh
# Wallet unit tests (derivation, persistence, locking)
cargo test -p epic_snip_wallet

# Chain-level recovery and hardening tests
cargo test -p epic_chain --test snip_wallet_recovery -- --test-threads=1

# Step 3D demonstration (3 payments, claims, conflicts, restart)
cargo test -p epic_chain --test snip_wallet -- --test-threads=1

# Core cryptographic and consensus tests
cargo test -p epic_core --lib

# Spend attribution and authorization tests
cargo test -p epic_chain --test spend_attribution -- --test-threads=1

# Receipt claim roundtrip
cargo test -p epic_chain --test receipt_claim_roundtrip -- --test-threads=1
```

## Limitations

- **Research prototype**: not for production deployment.
- Epic Cash does not support this mechanism without the implemented extensions.
- No production network activation is intended.
- Alice can reclaim an unspent Receipt at any time.
- Receipt discovery is not equivalent to final payment settlement.
- Alice seed-only recovery requires re-importing the public descriptor.
- The security review is not a formal cryptographic proof.
- The prototype does not claim production readiness.
- Activation is gated by test flags, not height-based consensus rules.

## Project Conclusion

**Epic SNIP demonstrates that semi-non-interactive payments can be implemented
using targeted extensions to Epic Cash's existing Mimblewimble cryptographic and
consensus architecture.**

The objective was technical feasibility, not production deployment.

## Further Documentation

- [PROTOCOL.md](PROTOCOL.md) — functional specification
- [POC_RESULTS.md](POC_RESULTS.md) — PoC validation results
- [CURRENT_STEP.md](CURRENT_STEP.md) — implementation history (Steps 3A–3H)
- [docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md](docs/SNIP_SPEND_AUTHORIZATION_DESIGN.md) — design and threat model
- [docs/SNIP_SPEND_ATTRIBUTION_RESEARCH.md](docs/SNIP_SPEND_ATTRIBUTION_RESEARCH.md) — attribution research

---

# Epic SNIP — One-Sided Reusable Payments

> **Experimental research state — not for real funds.** The local
> `upstream/epic` fork contains a Usernet-only consensus draft and is neither
> compatible with Epic Mainnet/Floonet nor cryptographically audited. It must
> not be pushed to EpicCash upstreams and must not be understood as
> production-ready.

## Top Priority

This project pursues **one** thing: the complete demonstration of the
transaction flow defined in [PROTOCOL.md](PROTOCOL.md).

Bob publishes a **Payment Descriptor** once.
Alice creates **three independent payments** from it
(without Bob, without communication) and brings them to
confirmation in the local **Usernet chain**.
Bob **detects** all three **by scanning the chain**,
determines **that** and **the amount**, and **claims** each
in a **separate transaction** to an output
**exclusively spendable by him**.
All three claims are confirmed in the Usernet chain.

**Additional requirements (PROTOCOL.md §3.8, §3.9):**
- **Alice can verify from the chain** that Bob has
  taken over the payments.
- **Alice can claim the shared outputs herself at any time**,
  as long as Bob has not already transferred them into his
  own outputs. The first confirmed valid spend wins
  (First-Confirmed-Spend-Wins).

All other work in this repo — the in-memory POC under
`src/one_sided.rs`, the receipt-claim research path under
`upstream/epic`, the attribution research — is
**completed and superseded by this specification**.
It remains as a historical record.

## Running

```sh
cargo fmt --check
cargo test
```

### Target test (in progress)

The acceptance integration test to be written must demonstrate the
complete 3-payment flow:

```sh
# (not yet available — follows from PROTOCOL.md §5)
cargo test -p epic_chain --test snip_acceptance
```

### Existing, completed tests (historical)

```sh
cd upstream/epic
cargo test -p epic_core --lib
cargo test -p epic_chain --test receipt_claim_roundtrip
cargo test -p epic_chain --test spend_attribution
cargo run -p epic_core --example receipt_stealth_demo
```

These tests serve research and verification of individual
building blocks. Per PROTOCOL.md §6 they **do not replace**
the complete 3-payment demonstration.

## Security Boundaries

No Mainnet compatibility. No real funds. No
networks. All keys are ephemeral test material.
The local range-proof, kernel-signature, and
TxHashset validation cover only the described
Usernet research scope.

## Documentation

- **Top priority and functional specification:** [PROTOCOL.md](PROTOCOL.md)
- **Ongoing status:** [CURRENT_STEP.md](CURRENT_STEP.md)
- Historical: [One-sided-POC](docs/one-sided-poc.md), [Receipt-Claim draft](docs/receipt-claim-design.md), [Cryptographic foundation](docs/cryptographic-foundation.md), [Protocol specification (POC)](docs/protocol-spec.md), [Threat model](docs/threat-model.md), [Assumptions](docs/assumptions-and-open-questions.md), [Test plan (POC)](docs/test-plan.md), [Continuation guide](docs/continuation-guide.md), [Usernet experiment](docs/usernet-experiment.md), [Spend-attribution research](docs/SNIP_SPEND_ATTRIBUTION_RESEARCH.md)

All documentation links to the historical paths
describe the **completed** research state, **not** the
target state defined in PROTOCOL.md.