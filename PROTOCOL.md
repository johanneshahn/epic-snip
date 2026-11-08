# Protocol: One-Sided Reusable Payments (SNIP)

Status: Functional specification. Not a consensus specification.
Not for Mainnet, Floonet, or real funds.

## Top Priority — Acceptance Criterion

Every further documentation, design decision, and implementation serves
this single acceptance criterion. Deviations from this criterion are only
permissible if they demonstrably lead to more compliance with §3.

**Bob publishes a Payment Descriptor exactly once.**
Then, without ever communicating with Bob or involving Bob's
participation, **Alice** creates **three independent payments**
and brings **all three** to confirmation in the local Usernet chain.
**Bob detects** all three payments **exclusively by scanning the chain**.
**Bob claims** each of the three payments in **his own transaction**.
**All three claims** are **confirmed in the Usernet chain**.
**The final outputs** are **exclusively spendable by Bob**.

Each individual requirement from §2 and §3 of the functional
specification additionally applies, as long as it does not
conflict with this acceptance criterion.

## §2 Transaction Flow (5 Steps)

1. **Bob: Payment Descriptor once.**
   Bob generates his long-term key material and publishes the public
   **Payment Descriptor**. The descriptor is reusable: Alice can derive
   arbitrarily many payments from it without involving Bob again.

2. **Alice: independent payments.**
   For each payment, Alice creates a separate transaction from the
   descriptor. Bob is not online, not involved in creation, and receives
   no message. Each payment is a separate, independent transaction.

3. **Alice: confirmation in the Usernet chain.**
   Alice brings each of the three payments into a block of the local
   Usernet chain and receives confirmation. Confirmation happens without
   Bob.

4. **Bob: detection by scanning.**
   Bob scans the Usernet chain. **Without any communication with Alice**
   he recognizes each of the three payments as addressed to him. From the
   chain he can derive:
   - *that* a payment is addressed to him,
   - *what amount* the payment has.
   Bob is offline during Alice's creation. Bob does not need to
   participate, *but* he must be able to **find** the payments.

5. **Bob: claim in his own transaction.**
   Bob claims each of the three payments in a separate transaction to an
   output that is **exclusively spendable by Bob**. The claim contains
   **no** Alice participation. All three claims are confirmed in the
   Usernet chain. The final outputs belong exclusively to Bob.

## §3 Binding Requirements

| No | Requirement |
| --- | --- |
| 3.1 | Bob must be able to **recognize** incoming payments without communicating with Alice. |
| 3.2 | Bob must be able to **determine the amounts** of payments addressed to him. |
| 3.3 | Bob must be able to claim payments **without Alice**. |
| 3.4 | The final outputs must be **exclusively spendable by Bob**. |
| 3.5 | All transactions must satisfy the **Epic Cash consensus** of the local Usernet chain. |
| 3.6 | **Confidentiality and privacy** must be preserved according to the EPIC project. |
| 3.7 | **No superfluous publicly linkable information** may be left on the chain. |
| 3.8 | **Alice can verify that Bob has taken over the shared outputs.** Alice must be able to follow from the chain that Bob has claimed a payment and that the funds now reside in his final outputs. |
| 3.9 | **Alice can claim the shared outputs herself at any time, as long as Bob has not yet transferred them into his own outputs.** The Alice-reclaim path is a competing path to the Bob-claim. The first confirmed valid spend wins (First-Confirmed-Spend-Wins). |

## §4 Not Binding — Free Design Decisions

The following points are **deliberately not** prescribed by the
functional specification. Any design decision regarding these points is
permissible, as long as it satisfies §2, §3, and §5:

- Shared outputs / split outputs
- The concrete key derivation
- Additional kernel features
- Consensus extensions beyond what §3.5 requires

**No longer free:** The `BobClaim` kernel is **part of the target protocol**
(§3.3). The Alice-reclaim path uses the **existing standard mechanism of
Epic Cash** (ordinary `Plain` kernel) and does not need to be reinvented
(§3.9).

## §5 Acceptance Criterion (verbatim)

> "The complete demonstration of the transaction flow:
> Bob creates the descriptor once → Alice creates 3
> independent payments (Bob not involved) → all 3
> are confirmed in the local Usernet chain → Bob
> detects all 3 → Bob claims each through his own
> transaction → all 3 claims are confirmed → final outputs
> exclusively Bob's."
>
> Additionally:
> - Alice can verify from the chain that Bob has
>   taken over the payments (§3.8).
> - Alice can claim the shared outputs herself at any time,
>   as long as Bob has not already transferred them
>   (§3.9).

## §6 Principle

> "A successful isolated crypto or consensus test
> **does not replace** the complete demonstration of the
> transaction flow."
>
> Every single component — crypto, consensus, wallet scan,
> claim — must demonstrably work **within the complete
> 3-payment demonstration**.

## Status Term

All documents in this repo that refer to the *previous*
research (one-sided shared-redeemable POC,
`AliceReclaim`, `BobClaim`-attribution research) are
**completed and superseded by this specification**.
They remain as historical records; their content is
**not** guidance for further development.