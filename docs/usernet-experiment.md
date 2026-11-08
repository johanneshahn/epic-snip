# Isolated Epic Usernet for SNIP Experiments

Status: Infrastructure design. This setup uses only the local
`upstream/epic` subproject and does not connect to Mainnet or Floonet.
It contains a local, experimental receipt-claim consensus path. This
is intended for Usernet only, uncommitted, unaudited, and neither
network- nor mainnet-compatible.

> **Top priority.** The Usernet infrastructure serves the
> complete 3-payment demonstration per
> [PROTOCOL.md](../PROTOCOL.md) §5:
> Bob publishes a Payment Descriptor once,
> Alice creates three independent payments, all three
> are confirmed in the Usernet chain, Bob detects
> them by scanning, Bob claims each in his own
> transaction, and the final outputs are
> exclusively spendable by Bob.
>
> A successful isolated crypto or consensus test
> **does not replace** the complete
> 3-payment demonstration
> ([PROTOCOL.md §6](../PROTOCOL.md)).

## Existing Support

The existing node supports `--usernet`, which internally
selects `ChainTypes::UserTesting`. Local standard endpoints
are provided for this:

## Terminal Demo without Wallet

The local core contains a direct terminal demo for the
stealth-receipt derivation path. It does not create a block,
but builds a valid local core transaction: Alice creates a
Receipt output with a valid range proof; Bob scans the hint,
derives the shared blind and claim key, rewinds the amount,
and builds a one-input/one-output BobClaim. Full
`Transaction::validate(...)` verification runs in Usernet mode:

```sh
cargo run -p epic_core --example receipt_stealth_demo
```

This is not a replacement for wallet scan, UTXO/TxHashset consumption,
or Usernet block end-to-end tests.

The core builder `libtx::build::receipt_output(...)` still only checks
the Receipt output format with a normal keychain blind. The terminal
demo additionally uses the builders clearly marked as research APIs
for the shared-blind output and the BobClaim transaction. They are
not wallet- or network-ready.

| Service | Default |
| --- | --- |
| Node API | `127.0.0.1:23413` |
| P2P | `127.0.0.1:23414` |
| Wallet listener | `127.0.0.1:23415` |
| Stratum | `127.0.0.1:23416` |

`run_test_miner` is explicitly intended for `usertesting`. For real
coinbase spending it needs a running local wallet listener; without
one no valuable data should be expected.

## Setup

From the project root:

```sh
mkdir -p .usernet/node
cd .usernet/node
cargo run --manifest-path ../../upstream/epic/Cargo.toml -- --usernet server config
```

In the generated `epic-server.toml`, set exclusively for this local
Usernet:

```toml
run_test_miner = true
test_miner_wallet_url = "http://127.0.0.1:23415"
```

Then start node and local wallet listener in separate terminals. The
exact wallet command depends on the checked-in `epic-wallet` CLI state
and must be verified with `cargo run -- --help` before execution.
Afterwards:

```sh
cd .usernet/node
cargo run --manifest-path ../../upstream/epic/Cargo.toml -- \
  --usernet server --config_file epic-server.toml run
```

The node must only bind to `127.0.0.1` and never be started with a
Mainnet configuration. Data under `.usernet/` is development data, not
a wallet backup.

## Planned Validation

After the core extension, the Usernet must accept or reject at least
the following real blocks:

1. Create a special output with Bob's receipt-key hash.
2. Mine a valid `BobClaim` and later retrieve its kernel.
3. Reject a plain kernel for a special input in the mempool and during
   block processing.
4. Reject wrong receipt key, wrong signature, replay, and wrong
   commitment.

## Automated Local Chain Roundtrip (historical)

The test `chain/tests/receipt_claim_roundtrip.rs` covers a real
local chain flow without a network: it mints a mature Usernet test
coinbase, mines Alice's receipt funding transaction, confirms the
receipt UTXO, mines Bob's claim in a follow-up block, and verifies
UTXO consumption, destination output, and full TxHashset validation.

```sh
cargo test -p epic_chain --test receipt_claim_roundtrip
```

The test starts no node, wallet, or mempool. It therefore does not
replace a multi-process Usernet test and proves neither P2P nor
Mainnet compatibility. **It covers exactly one payment and is therefore
not the acceptance demonstration required by PROTOCOL.md §5.**

## Acceptance Test (in progress)

The integration test to be written,
`chain/tests/snip_acceptance.rs`, must demonstrate the complete
3-payment flow:

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
   (any other spend attempt is rejected).

```sh
cargo test -p epic_chain --test snip_acceptance
```

Per [PROTOCOL.md §6](../PROTOCOL.md): a successful isolated crypto
or consensus test **does not replace** the complete demonstration
of the transaction flow.