# SNIP Privacy Research

## Overview

This document investigates whether the two independent spending rights
of Alice and Bob can be preserved without a blockchain observer
permanently recognizing which Receipt was spent, when, and via which
path — using only the cryptographic primitives already present in
Epic Cash.

Each conclusion is classified as one of:

- **Demonstrated**: empirically verified by existing tests.
- **Supported under explicit assumptions**: follows from stated
  cryptographic or architectural assumptions, but not empirically
  proven.
- **Unproven**: plausible but lacks supporting evidence or
  assumptions.
- **Incorrect**: contradicted by evidence or reasoning.

No code changes are made in this phase.

---

## Question 1: Can an observer with Bob's public descriptor determine which Receipts are addressed to Bob?

### Setup

Bob's public descriptor: `S = sG` (scan public), `P = pG` (receipt spend public).

Alice creates payment i:
- `r_i` = fresh ephemeral secret (Alice-only)
- `R_i = r_i * G` (public hint, stored in Receipt output)
- `h_i = H(domain || ECDH(r_i, S) || R_i || descriptor_id)`
  where `ECDH(r_i, S) = r_i * s * G`
- `B_i = P + h_i * G` (Bob's one-time receipt key, public only after claim)
- `hashes.bob = H(domain || B_i)` (stored in Receipt output)

The observer knows: `S`, `P`, `R_i`, `hashes.bob`, `C_i`, `hashes.alice`.

### Analysis

To determine whether a Receipt output is addressed to Bob, the observer
must verify that `hashes.bob == H(P + h_i * G)` for some valid `h_i`.
Computing `h_i` requires `ECDH(r_i, S) = r_i * s * G`.

The observer knows `R_i = r_i * G` and `S = s * G`. Computing
`r_i * s * G` from `r_i * G` and `s * G` is the **Computational
Diffie-Hellman (CDH) problem** on secp256k1. Under the standard CDH
assumption, this is computationally infeasible.

Therefore:
- The observer **cannot** compute `h_i` without Bob's private scan
  secret `s`.
- The observer **cannot** compute `B_i = P + h_i * G`.
- The observer **cannot** verify whether `hashes.bob` matches any
  Bob-derived key.

**Classification: Supported under explicit assumptions.** An observer
with only Bob's public descriptor cannot determine which Receipts are
addressed to Bob, under the standard Computational Diffie-Hellman (CDH)
assumption on secp256k1. The descriptor being public does not break
recipient privacy. This is not **Demonstrated** (no formal proof of CDH
hardness) and not **Infeasible** in an absolute sense — it is contingent
on the CDH assumption holding.

### Sub-question: Can Alice derive Bob's exclusive key `b_i`?

Alice knows `r_i` (she chose it), `S` (public), and `P` (public). She
can compute `ECDH(r_i, S) = r_i * s * G`? No — she does not know `s`.
She knows `r_i * G = R_i` (her own ephemeral public key) and `s * G = S`,
but computing `r_i * s * G` is CDH. She cannot compute `h_i` or `B_i`.

Even though Alice knows `r_i`, she cannot derive `b_i = p + h_i`
because she lacks `p` (Bob's private receipt spend secret) and cannot
compute `h_i` (requires `s`).

**Classification: Supported under explicit assumptions.** Alice cannot
derive Bob's exclusive claim key `b_i`. This follows from the CDH
assumption (Alice cannot compute `ECDH(r_i, S)` without `s`) and the
discrete logarithm assumption (Alice cannot recover `p` from `P = pG`).
It is not **Demonstrated** (no formal proof) but is well-supported by
standard cryptographic assumptions.

### Sub-question: Can Bob derive Alice's exclusive key `a_i`?

Bob knows `s` (his scan secret) and can compute `ECDH(s, R_i) = r_i * s * G`,
hence `h_i` and `B_i`. But `a_i` is derived from Alice's seed via a
separate hardened path (`ROLE_ALICE`, `KEY_ALICE_AUTH`). Bob has no
access to Alice's seed. The shared blind `q_i` is derived from the
ECDH shared secret but is independent of `a_i`.

Even though Bob can compute `q_i` (the shared blind), he cannot derive
`a_i = KDF(Alice_seed || index)` because he lacks Alice's seed.

**Classification: Supported under explicit assumptions.** Bob cannot
derive Alice's exclusive reclaim key `a_i`. This follows from the
discrete logarithm assumption (Bob cannot recover `a_i` from
`A_i = a_i * G`) and the fact that `a_i` is derived from Alice's seed
via a separate hardened path that Bob has no access to. The shared blind
`q_i` is independent of `a_i`. It is not **Demonstrated** but is
well-supported by standard assumptions.

### Important limitation and distinction between observer types

**Passive observer (knows only Bob's public descriptor):**
Under the CDH assumption, the observer cannot compute `h_i` or `B_i`
and therefore cannot identify which Receipts are addressed to Bob. The
public descriptor alone does not enable recipient identification. This
is **Supported under explicit assumptions** (CDH).

**Alice (knows `r_i`, the ephemeral secret she chose):**
Alice knows `r_i` but still cannot compute `ECDH(r_i, S) = r_i * s * G`
because she lacks `s`. She can compute `B_i` only if she knows `h_i`,
which requires `s`. Therefore Alice also cannot derive `b_i` or identify
Bob's Receipts beyond those she created herself. This is **Supported
under explicit assumptions** (CDH + discrete log).

**Bob (knows `s`, his scan secret):**
Bob can compute `ECDH(s, R_i) = r_i * s * G`, derive `h_i` and `B_i`,
and verify `hashes.bob == H(B_i)`. Bob can identify his Receipts. This
is **Demonstrated** (implemented and tested in `scan_incoming`).

**Key distinction:** The descriptor being public is not itself a privacy
weakness. The security of recipient privacy rests on the CDH assumption,
not on descriptor secrecy. However, this is a cryptographic assumption,
not a proof. If CDH were broken, recipient privacy would be lost. The
classification is therefore "Supported under explicit assumptions," not
"Demonstrated" or "Infeasible."

---

## Question 2: Which information is strictly necessary for permanent consensus validation?

### Receipt output (stored permanently in UTXO set or spent set)

| Field | Size | Needed for | Can it be pruned? |
|---|---|---|---|
| `commit` (Pedersen) | 33 bytes | UTXO identity, balance verification | No — core to transaction validity |
| `proof` (Bulletproof) | ~675 bytes | Range proof verification | No — required for amount confidentiality |
| `receipt_key_hashes.bob` | 32 bytes | Bob authorization verification | Only after confirmation horizon (see below) |
| `receipt_key_hashes.alice` | 32 bytes | Alice authorization verification | Only after confirmation horizon |
| `receipt_stealth_hint` | 33 bytes | UTXO identity, key derivation | Only after confirmation horizon |

### Spend evidence (currently exempt from cut-through)

| Field | Size | Needed for | Can it be pruned? |
|---|---|---|---|
| `BobClaim` kernel | ~200 bytes | Bob authorization proof | Only after confirmation horizon |
| `ReceiptSpendAuthorization` | ~160 bytes | Alice authorization proof | Only after confirmation horizon |
| `original_commitment` | 33 bytes | Links spend to Receipt | Only after confirmation horizon |
| `kernel_excess` | 33 bytes | Kernel binding | Only after confirmation horizon |
| `kernel_hash` | 32 bytes | Kernel binding | Only after confirmation horizon |

### Analysis

**Immediately necessary (cannot be pruned):**
- The Receipt output's `commit` and `proof` must remain in the UTXO set
  (if unspent) or in the spent set (for historical validation). These
  are required for balance verification and amount confidentiality.
- The UTXO identity (`features`, `commit`, `receipt_key_hashes`, `hint`)
  must be preserved for spend matching.

**Potentially prunable after a confirmation horizon:**
- The authorization evidence (`BobClaim` kernel, `ReceiptSpendAuthorization`,
  `original_commitment`, `kernel_excess`, `kernel_hash`) is needed for
  consensus validation at the time of the spend. After the spend is
  confirmed and a sufficient number of blocks have passed, the evidence
  could theoretically be pruned.

**However, pruning introduces a trust assumption:**
- New nodes joining the network must validate the chain from genesis.
  If evidence is pruned, they cannot independently verify historical
  spends.
- This is the **Horizon Attack** risk identified in LIP-0004: a deep
  reorg could revert a spend whose evidence has been pruned, and the
  node cannot re-validate.
- The LIP-0004 approach addresses this with a defined confirmation
  horizon and checkpoint trust, but acknowledges the risk.

**Classification: Supported under explicit assumptions.** Authorization
evidence could be pruned after a confirmation horizon, but this requires
checkpoint trust for new nodes (Horizon Attack risk). The current
implementation preserves all evidence (cut-through exemption) to
maintain full historical verifiability without trust assumptions. Whether
pruning is safe depends on the checkpoint trust model, which is an
architectural choice, not a cryptographic limitation. The current
design's permanence is a consequence of the cut-through exemption
decision, not a fundamental requirement of the protocol.

---

## Question 3: Could an aggregatable authorization balance replace permanent Receipt linking?

### The idea

Instead of binding each Receipt to a specific BobClaim or AliceSpend
(via `original_commitment` and key hash matching), use an additional
aggregatable balance equation (inspired by LIP-0004) where the
authorization is verified via a sum of blinds rather than per-input
matching.

### Requirements

1. Either Alice or Bob can authorize a spend (but not both simultaneously).
2. After Bob claims, Alice cannot spend (and vice versa).
3. Consensus can verify the authorization without permanent per-input
   evidence (or with delayed pruning).

### Analysis

**Current approach (SNIP):** Each Receipt input is bound to exactly one
resolution kernel via `original_commitment` matching and key hash
verification. The authorization is non-transferable: Alice's auth
cannot be used by Bob (different key, different hash) and vice versa.
This is enforced by the two-branch rule in `verify_receipt_resolution`.

**Aggregatable approach (LIP-0004 style):** LIP-0004 uses stealth
addresses with scan/spend keys, an additional balance equation
(sum of input blinds = sum of output blinds + fee), and delayed
cut-through. Aggregation hides individual transaction relationships.

**The challenge for SNIP:** The two-branch authorization (BobClaim OR
AliceSpend) requires consensus to distinguish which path was taken.
An aggregatable scheme would need an **OR-signature** (Bob key OR Alice
key) that is:
- Non-interactive (no communication between Alice and Bob).
- Verifiable by consensus.
- Non-transferable (Alice's signature cannot be reused by Bob).
- Aggregatable (can be combined with other transactions).

**Current Epic Cash primitives:** Epic uses AgSig (aggregated
signatures) which supports multi-signature aggregation but not
OR-composition. AgSig combines multiple signatures into one, but
requires all signers to participate. An OR-signature would allow
either signer to produce a valid signature independently.

**Classification: Supported under explicit assumptions.** The current
design specifically requires distinguishing which of two authorization
paths (BobClaim or AliceSpend) was taken for each Receipt input. This
is a disjunction (OR) verification. Epic Cash's AgSig supports
multi-signature aggregation (AND-composition) but not OR-composition.
An OR-signature or zero-knowledge proof would be needed to make the
authorization aggregatable without per-input matching.

However, this is an **implementation limitation** (no OR-signature
primitive available), not a fundamental cryptographic impossibility.
If Epic Cash were extended with OR-signatures or a ZK proof system,
an aggregatable construction might become feasible. The current
approach (per-input matching with non-transferable authorizations) is
the minimal construction satisfying the requirements using the
primitives actually available.

**Counterexample consideration:** Could a different construction avoid
the OR by using the shared blind `q_i`? Both parties know `q_i`, so
a spend authorized by `q_i` would not distinguish Alice from Bob.
The two-branch rule exists precisely because `q_i` is shared — without
independent authorization keys, either party could spend. Therefore,
some form of independent authorization (and hence a disjunction
check) is fundamentally required. The disjunction is necessary; only
the specific mechanism (per-input matching vs. OR-signature) is an
implementation choice.

---

## Test Catalog

| Test | What must be demonstrated | Status |
|---|---|---|
| Independent sending | Alice creates a valid payment without Bob | ✅ Demonstrated (phase_a) |
| Independent BobClaim | Bob can claim without Alice | ✅ Demonstrated (phase_c) |
| Independent AliceSpend | Alice can reclaim an unclaimed Receipt | ✅ Demonstrated (phase_c) |
| Double-spend protection | Both paths cannot succeed | ✅ Demonstrated (phase_d, phase_e) |
| Recipient privacy | Observer cannot group payments by public descriptor | **Supported under explicit assumptions** (see Q1) |
| Transaction unlinkability | Aggregation hides original payment relationships | Not Yet Tested |
| Cut-through safety | Pruning does not create invalid/unauthorized spends | Plausible (by design) |
| Historical validation | New nodes can safely verify chain state | ✅ (evidence preserved) |

## Simulation Note

Simulated transaction data could be used to measure a passive
observer's ability to correlate Receipts with claims. The hit rate
could be compared with ordinary Epic transactions. However, such
simulations are useful but do not replace a cryptographic security
proof.

## Scientific Basis

Two works are relevant:

1. **Fuchsbauer and Orrù, "Non-interactive Mimblewimble transactions,
   revisited"** — investigates the security and privacy of
   non-interactive Mimblewimble transactions and formulates
   corresponding security arguments.

2. **Litecoin LIP-0004** — concrete construction with stealth keys,
   additional balance equation, aggregation, and delayed cut-through.
   The specification also describes a possible Horizon Attack with deep
   reorganizations.

## Classification Summary

| Finding | Classification | Basis |
|---|---|---|
| Observer cannot identify Bob's Receipts from descriptor | Supported under explicit assumptions | CDH assumption on secp256k1 |
| Alice cannot derive `b_i` | Supported under explicit assumptions | CDH + discrete log assumptions |
| Bob cannot derive `a_i` | Supported under explicit assumptions | Discrete log + seed separation |
| Authorization evidence prunable after horizon | Supported under explicit assumptions | Checkpoint trust model (architectural) |
| Aggregatable authorization with current primitives | Supported under explicit assumptions | No OR-signature primitive available |
| Cut-through exemption creates permanent linkability | Demonstrated | By design (code inspection) |
| Amount hiding | Demonstrated | Pedersen commitments + Bulletproofs |
| Identity separation | Demonstrated | Independent key derivation |
| Recipient privacy (observer grouping) | Supported under explicit assumptions | CDH assumption |

## Conclusion

The three research questions yield the following assessments:

| Question | Classification | Key reason |
|---|---|---|
| 1. Can observer with descriptor identify Bob's Receipts? | **Supported under explicit assumptions** | Requires solving CDH; descriptor being public does not break recipient privacy |
| 2. Can authorization evidence be pruned after a horizon? | **Supported under explicit assumptions** | Requires checkpoint trust for new nodes (Horizon Attack risk) |
| 3. Can aggregatable authorization replace permanent linking? | **Supported under explicit assumptions** | Requires OR-signatures not present in Epic Cash; disjunction is necessary, mechanism is implementation choice |

**Most important finding:** The privacy limitation that is truly
unavoidable is the permanent structural linkability created by the
cut-through exemption. This is not a weakness of the key derivation
but a consequence of the two-branch enforcement requirement. Whether
this linkability can be reduced depends on whether new cryptographic
primitives (OR-signatures, delayed pruning with checkpoint trust)
can be integrated.

**Key distinction:** The disjunction (OR) between BobClaim and
AliceSpend is fundamentally required — without independent
authorization keys, either party could spend. The specific mechanism
(per-input matching vs. OR-signature) is an implementation choice
constrained by available primitives. The permanent linkability is a
consequence of the current implementation's choice to preserve
evidence for historical validation, not a fundamental requirement of
the two-branch model.

The research confirms that SNIP's privacy properties are sound at
the protocol level (amount hiding, identity separation) but that the
cut-through exemption creates a durable, publicly verifiable footprint
that is a trade-off of the current design, not necessarily a
fundamental limitation.