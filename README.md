# Epic SNIP POC

Ein deterministischer In-Memory-Forschungsprototyp für eine hypothetische Zahlung mit zwei Ausgabepfaden: Bob kann jederzeit claimen; Alice kann ab einer simulierten Blockhöhe refunden. Die Bibliothek speichert weder Daten noch verwendet sie Netzwerk oder Systemzeit.

## Ausführen

```sh
cargo fmt --check
cargo test
```

Die kleine öffentliche API besteht aus `Ledger::create_payment`, `claim`, `refund` und `advance_height`. Schlüssel in `test_support` sind ausschließlich für Tests.

## Sicherheitsgrenzen

Dies ist **keine Epic-Cash-Wallet, keine Blockchain und nicht konsenskompatibel**. Es gibt keine realen Schlüssel, Gelder oder Netzwerke. Der `commitment_placeholder` ist lediglich ein kanonisch gehashter Testwert; er ist kein Pedersen Commitment und bietet weder Betragsvertraulichkeit noch Konsenssicherheit.

Es fehlen insbesondere Range Proofs, Kernel-Signaturen, Cut-through, Wallet-Interaktivität und eine Mimblewimble-Transaktion. Es gibt keine formale Sicherheitsanalyse und keinen kryptografischen Beweis. Die Tests belegen ausschließlich Zustände dieser Simulation. Jede produktive Verwendung erforderte eine formale Spezifikation, ein Konsensdesign sowie unabhängige kryptografische und Implementierungs-Audits.

Weitere Details: [Protokollspezifikation](docs/protocol-spec.md), [Bedrohungsmodell](docs/threat-model.md), [Annahmen](docs/assumptions-and-open-questions.md) und [Testplan](docs/test-plan.md).
