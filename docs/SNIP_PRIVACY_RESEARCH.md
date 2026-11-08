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

**Passive observer:** The observer knows `R_i = r_i * G` (public hint)
and `S = s * G` (public descriptor). Computing `r_i * s * G` from
`r_i * G` and `s * G` is the **Computational Diffie-Hellman (CDH)
problem** on secp256k1. Under the standard CDH assumption, this is
computationally infeasible.

Therefore (for a passive observer):
- The observer **cannot** compute `h_i` without solving CDH.
- The observer **cannot** compute `B_i = P + h_i * G`.
- The observer **cannot** verify whether `hashes.bob` matches any
  Bob-derived key.

**Alice (who knows `r_i`):** Alice CAN compute `ECDH(r_i, S) = r_i * s * G`
because she knows `r_i` and `S`. She can compute `h_i` and
`B_i = P + h_i * G`. However, she cannot derive `b_i = p + h_i`
because she lacks `p` (Bob's private receipt spend secret). Alice's
ability to compute `h_i` does not enable her to forge Bob's claim or
identify Bob's Receipts beyond those she created.

**Classification: Supported under explicit assumptions.** A passive
observer with only Bob's public descriptor cannot determine which
Receipts are addressed to Bob, under the standard Computational
Diffie-Hellman (CDH) assumption on secp256k1. The descriptor being
public does not break recipient privacy. Alice can compute the ECDH
shared secret but cannot derive Bob's private key. This is not
**Demonstrated** (no formal proof of CDH hardness) and not
**Infeasible** in an absolute sense — it is contingent on the CDH
assumption holding.

### Sub-question: Can Alice derive Bob's exclusive key `b_i`?

**Correction:** Alice CAN compute the ECDH shared secret. She knows her
ephemeral private key `r_i` and Bob's public scan key `S = sG`. By the
fundamental property of ECDH, she computes:

`ECDH(r_i, S) = r_i * s * G`

This is the same value Bob computes as `ECDH(s, R_i) = s * r_i * G`.
Both arrive at the identical shared secret.

Alice can therefore compute:
- `h_i = H(domain || r_i * s * G || R_i || descriptor_id)` — she knows
  all inputs (the shared secret, her own `R_i`, and the public
  descriptor_id).
- `B_i = P + h_i * G` — she knows `P` (public) and `h_i`.

**However, Alice still cannot derive `b_i`.** Bob's private receipt key
is:

`b_i = p + h_i`

where `p` is Bob's private receipt spend secret. Alice knows `h_i` but
does not know `p`. She cannot recover `p` from `P = pG` (discrete
logarithm problem). Therefore she cannot compute `b_i = p + h_i`.

Alice's capability: she can compute `B_i` (the public key) and verify
that a BobClaim's `receipt_public_key` matches `B_i`, confirming Bob
claimed her payment. But she cannot forge a BobClaim signature because
that requires `b_i` (the private key).

**Classification: Supported under explicit assumptions.** Alice cannot
derive Bob's exclusive claim key `b_i`. This follows from the discrete
logarithm assumption (Alice cannot recover `p` from `P = pG`), not
from CDH. Alice CAN compute the ECDH shared secret and `h_i` — the
barrier is the discrete logarithm problem on `p`, not CDH. It is not
**Demonstrated** (no formal proof) but is well-supported by standard
cryptographic assumptions.

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

### Sub-question: Does knowledge of one payment's ephemeral secret or shared secret enable linking other Receipts?

If an attacker learns `r_i` for one payment (e.g., through device
compromise), they can compute `h_i` and `B_i` for that payment. They
can identify that specific Receipt as one created by Alice (since Alice
created it). However:

- They cannot compute `h_j` or `B_j` for other payments (different
  `r_j`, different shared secret).
- They cannot recover Bob's private scan secret `s` from the shared
  secret `r_i * s * G` (discrete logarithm on `s`).
- They cannot recover Bob's private receipt spend secret `p` from `P`.

Therefore, compromising one payment's ephemeral secret does not enable
linking other Receipts. The attacker gains knowledge of one payment only.

**Classification: Supported under explicit assumptions** (discrete log
assumption on `s` and `p`).

### Important limitation and distinction between observer types

**Passive observer (knows only Bob's public descriptor):**
Under the CDH assumption, the observer cannot compute `h_i` or `B_i`
and therefore cannot identify which Receipts are addressed to Bob. The
observer knows `R_i = r_i * G` and `S = s * G` but cannot compute
`r_i * s * G` (CDH). Without `h_i`, the observer cannot compute
`B_i = P + h_i * G` or verify `hashes.bob == H(B_i)`. The public
descriptor alone does not enable recipient identification. This is
**Supported under explicit assumptions** (CDH).

**Alice (knows `r_i`, the ephemeral secret she chose):**
Alice CAN compute the ECDH shared secret `r_i * s * G` because she
knows `r_i` and `S = s * G`. She can therefore compute `h_i` and
`B_i = P + h_i * G`. However, she still cannot derive `b_i = p + h_i`
because she does not know `p` (Bob's private receipt spend secret).
Alice can verify that a BobClaim's `receipt_public_key` matches her
computed `B_i`, confirming Bob claimed her payment (§3.8), but she
cannot forge a BobClaim signature. This is **Supported under explicit
assumptions** (discrete log on `p`).

**Bob (knows `s`, his scan secret):**
Bob can compute `ECDH(s, R_i) = r_i * s * G`, derive `h_i` and `B_i`,
and verify `hashes.bob == H(B_i)`. Bob can identify his Receipts. This
is **Demonstrated** (implemented and tested in `scan_incoming`).

**Key distinction:** The descriptor being public is not itself a privacy
weakness. Recipient privacy rests on the CDH assumption (for passive
observers) and the discrete logarithm assumption (for Alice). However,
these are cryptographic assumptions, not proofs. If CDH were broken,
recipient privacy would be lost. The classification is therefore
"Supported under explicit assumptions," not "Demonstrated" or
"Infeasible."

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
| Alice cannot derive `b_i` | Supported under explicit assumptions | Discrete log assumption on `p` (Alice CAN compute ECDH) |
| Bob cannot derive `a_i` | Supported under explicit assumptions | Discrete log + seed separation |
| Authorization evidence prunable after horizon | Supported under explicit assumptions | Checkpoint trust model (architectural) |
| Aggregatable authorization with current primitives | Supported under explicit assumptions | No OR-signature primitive available |
| Cut-through exemption creates permanent linkability | Demonstrated | By design (code inspection) |
| Amount hiding | Demonstrated | Pedersen commitments + Bulletproofs |
| Identity separation | Demonstrated | Independent key derivation |
| Recipient privacy (observer grouping) | Supported under explicit assumptions | CDH assumption |

## Controlled Privacy Audit: Descriptor Reuse Linkability

### Audit Objective

Determine whether a passive blockchain observer can identify that
multiple Receipt outputs belong to the same Bob, even when Bob's
identity is unknown. Specifically: does reusing the same public
recipient descriptor create any observable relationship between
otherwise independent Receipts?

### Experiment Details

**Test command:**
```sh
cd upstream/epic && cargo test -p epic_chain --test snip_wallet_recovery privacy_audit_descriptor_reuse -- --test-threads=1
```

**Sample size:** 6 Receipts total — 3 to Bob A (same descriptor), 3 to Bob B
(different descriptor). 2 BobClaims confirmed (1 per Bob). 4 Receipts
left unclaimed.

**Methodology:**
1. Create two distinct Bob descriptors (Bob A, Bob B) from different seeds.
2. Alice creates 3 Receipts to Bob A and 3 to Bob B using fresh ephemeral
   keys for every payment (different amounts).
3. Mine all 6 funding transactions into separate blocks.
4. Bob A claims 1 Receipt, Bob B claims 1 Receipt.
5. Extract all publicly observable fields from every Receipt output and
   BobClaim kernel on the chain: `commit`, `hint` (ephemeral public key),
   `receipt_key_hashes.bob`, `receipt_key_hashes.alice`, and BobClaim
   `receipt_public_key`.
6. Check for any duplicate or correlatable values across the 6 Receipts.
7. Check for any duplicate BobClaim public keys.
8. Compare groupings against known recipients (only after analysis).

**Observations:**
- All 6 Receipt commits are distinct.
- All 6 ephemeral hints (`R_i = r_i * G`) are distinct.
- All 6 `receipt_key_hashes.bob` values are distinct.
- All 6 `receipt_key_hashes.alice` values are distinct.
- The 2 BobClaim `receipt_public_key` values are distinct.
- No field value repeats across any two Receipts, regardless of recipient.
- No correlation between Receipts to the same Bob vs. different Bobs is
  observable from public fields alone.

**Result:** PASS — no detectable correlation found.

### Implementation Inspection

All publicly visible fields of a Receipt output and BobClaim kernel
were traced through the implementation:

| Field | Source | Repeats across same Bob? | Linkable? |
|---|---|---|---|
| `features` | `OutputFeatures::Receipt` (constant) | Yes (all Receipts) | No — type marker only |
| `commit` | `secp.commit(value, q_i)` | No — `q_i` unique per payment | No |
| `proof` | Bulletproof with `q_i` | No — depends on `q_i` and value | No |
| `receipt_key_hashes.bob` | `H(B_i)` where `B_i = P + h_i * G` | No — `h_i` unique per payment | No |
| `receipt_key_hashes.alice` | `H(A_i)` where `A_i = a_i * G` | No — `a_i` unique per payment/index | No |
| `receipt_stealth_hint` | `R_i = r_i * G` | No — `r_i` fresh per payment | No |
| BobClaim `receipt_public_key` | `B_i` (revealed at claim) | No — unique per payment | No |
| BobClaim `receipt_signature` | `Sign_{b_i}(...)` | No — `b_i` unique per payment | No |
| BobClaim `original_commitment` | `C_i` | Links claim to its Receipt only | No cross-Receipt link |

**Key finding: no on-chain field is identical or directly correlatable
across multiple Receipts to the same Bob.** The `descriptor_id` is
stored only in the wallet store, not on-chain.

### Cryptographic Analysis

**Passive observer without descriptor:**
The observer sees `hashes.bob = H(B_i)` where `B_i = P + h_i * G`
and `h_i = H(domain || ECDH(r_i, S) || R_i || descriptor_id)`.
The observer cannot compute `h_i` (requires `r_i * s * G` = CDH) and
cannot compute `B_i`. Each `hashes.bob` is a hash of a unique
one-time key. No correlation is possible.

**Passive observer with Bob's public descriptor (S, P):**
Even knowing `S` and `P`, the observer still cannot compute `h_i`
(CDH on `r_i * s * G`). They cannot verify that `hashes.bob` was
derived from this descriptor. They cannot link Receipts by descriptor.
The descriptor being public does not enable recipient identification.

**After Bob claims (B_i revealed):**
The observer sees `B_i` and can verify `hashes.bob == H(B_i)` (trivially,
both public). However:
- `B_i = P + h_i * G` where `h_i` is unknown (DLP on `B_i - P`).
- The observer cannot extract `h_i` from `B_i`.
- Different claims reveal different `B_i` values.
- No link between claims beyond "both are BobClaims" is observable.

**Knowledge of one payment's secrets:**
If an attacker learns `r_i` for one payment, they can compute `h_i`
and `B_i` for that payment only. They cannot compute `h_j` for other
payments (different `r_j`). They cannot recover `s` from `r_i * s * G`
(DLP). Therefore, compromising one payment does not enable linking
other Receipts.

### Potential Side Channels (Not Cryptographic)

The following are wallet-level or network-level concerns, not
protocol-level linkability:

1. **Transaction graph:** If Alice creates multiple Receipts in the
   same transaction or from the same input, an observer can correlate
   them via the transaction structure. Mitigation: separate transactions,
   coin selection.
2. **Timing:** Receipts created or claimed in quick succession may be
   correlated by timing. Mitigation: randomized delays.
3. **Amount patterns:** Distinctive amounts (e.g., 10, 20, 30) may
   correlate Receipts. Mitigation: use varied, non-distinctive amounts.
4. **Network layer:** IP addresses, broadcast timing. Mitigation: Tor,
   relays (out of scope).
5. **Descriptor publication:** If Bob's descriptor is published alongside
   his identity, an observer learns that Bob accepts payments, but
   cannot identify which on-chain Receipts are addressed to him (CDH).
   External information (e.g., Alice revealing she paid Bob, or timing
   correlation) could link a Receipt to Bob independently of the
   cryptographic construction. This is a descriptor-distribution and
   usage-pattern concern, not a protocol-level cryptographic weakness.

### Evaluation

1. **Receipts using the same descriptor contain no repeated or
   correlatable public information.** Each field is unique per payment
   due to fresh ephemeral keys and derived one-time keys. ✅ Verified
   by implementation inspection.

2. **Receipts cannot be grouped without knowing the recipient's
   identity.** No on-chain field repeats or correlates across Receipts
   to the same Bob. ✅ Verified.

3. **Knowing Bob's public descriptor does not change the result.** Even
   with the descriptor, the observer cannot compute `h_i` (CDH) and
   cannot link Receipts. ✅ Verified.

4. **Observing a BobClaim does not make other Receipts identifiable.**
   The revealed `B_i` is a one-time key. Different claims have different
   `B_i`. No cross-claim linkage is possible. ✅ Verified.

5. **Knowledge of one payment's ephemeral secret does not enable
   linking other Receipts.** Each payment uses an independent `r_i`.
   Compromising one does not reveal `s` or enable computing other
   payments' shared secrets. ✅ Verified.

6. **No observed correlations arise from cryptographic structure.**
   All observed correlations (if any) would be from metadata, timing,
   or test artifacts — not from the cryptographic construction. ✅ Verified.

### Conclusion

**Under the CDH and discrete logarithm assumptions, reusing a single
public Bob descriptor does NOT create a detectable relationship
between different SNIP Receipts.** No on-chain field repeats or
correlates across Receipts to the same Bob. The only potential
linkability is through wallet-level or network-level side channels
(transaction graph, timing, amounts), which are standard blockchain
privacy concerns, not SNIP-specific vulnerabilities.

This is **Supported under explicit assumptions** (CDH + discrete log),
not **Demonstrated** (no formal proof) and not **Infeasible** (contingent
on standard assumptions).

---

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