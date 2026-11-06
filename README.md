# Epic SNIP POC

Ein In-Memory-Forschungsprototyp für zwei hypothetische Modelle: den älteren
Claim/Refund-Zustandsautomaten und einen neuen **one-sided shared-redeemable**-Ablauf.
Beim letzteren veröffentlicht Bob einen Scan-Schlüssel. Alice erzeugt für jede
Zahlung einen Ephemeralschlüssel; Bob erkennt die separat zugestellte
Wallet-Nachricht und **muss** ihn auf einen neuen eigenen Schlüssel übertragen.
Erst die bestätigte Übertragung ist der Abschluss des semi-interaktiven
Transfers. Alice behält bewusst ebenfalls die Spend-Fähigkeit, bis Bob diesen
Schritt bestätigt hat. Die Bibliothek speichert
weder Daten noch verwendet sie Netzwerk oder Systemzeit.

## Ausführen

```sh
cargo fmt --check
cargo test
```

Die ältere API besteht aus `Ledger::create_payment`, `claim`, `refund` und
`advance_height`. Der neue POC verwendet `ReceiverKey`,
`OneSidedPayment::create`, `ReceiverKey::scan` und `OneSidedLedger`. Die
Bezeichnung `PendingEnvelope` ist ein technischer Name für die Wallet-Nachricht,
nicht ein behaupteter Konsensstatus.
Schlüssel in `test_support` sind ausschließlich für Tests.

## Sicherheitsgrenzen

Dies ist **keine Epic-Cash-Wallet, keine Blockchain und nicht konsenskompatibel**. Es gibt keine realen Schlüssel, Gelder oder Netzwerke. Der `commitment_placeholder` ist lediglich ein kanonisch gehashter Testwert; er ist kein Pedersen Commitment und bietet weder Betragsvertraulichkeit noch Konsenssicherheit.

Der neue Ablauf leitet eine gemeinsame Spend-Fähigkeit per ECDH ab und nutzt
Pedersen Commitments und AgSig nur als isolierte Primitiven. Sein
`PendingEnvelope` ist **kein** Epic-Output, kein Slate, Kernel oder Range Proof
und wird auch nicht in die vorhandene 20-Byte-Range-Proof-Nachricht gepackt.
Es fehlen insbesondere Range Proofs, Kernel-Signaturen, Cut-through, eine
gültige Mimblewimble-Transaktion, Netzwerk- und Mempool-Semantik. Es gibt
keine formale Sicherheitsanalyse und keinen kryptografischen Beweis. Die Tests
belegen ausschließlich Zustände dieser Simulation. Jede produktive Verwendung
erforderte eine formale Spezifikation, ein Konsensdesign sowie unabhängige
kryptografische und Implementierungs-Audits.

Weitere Details: [One-sided-POC](docs/one-sided-poc.md), [Receipt-Claim-Entwurf](docs/receipt-claim-design.md), [Kryptografische Grundlage](docs/cryptographic-foundation.md), [Protokollspezifikation](docs/protocol-spec.md), [Bedrohungsmodell](docs/threat-model.md), [Annahmen](docs/assumptions-and-open-questions.md), [Testplan](docs/test-plan.md) und die [Übergabe zur Weiterarbeit](docs/continuation-guide.md).

Die geplante lokale Chain-Validierung ist in der
[Usernet-Experimentanleitung](docs/usernet-experiment.md) dokumentiert.
