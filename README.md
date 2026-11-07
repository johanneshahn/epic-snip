# Epic SNIP POC

> **Experimenteller Forschungsstand — nicht für reale Gelder.** Der lokale
> `upstream/epic`-Fork enthält einen Usernet-only Konsensentwurf und ist weder
> mit Epic Mainnet/Floonet kompatibel noch kryptografisch auditiert. Er darf
> nicht an EpicCash-Upstreams gepusht und nicht als produktionsreif verstanden
> werden.

Ein In-Memory-Forschungsprototyp für einen **one-sided shared-redeemable**-
Ablauf. Bob veröffentlicht einen Scan-Schlüssel. Alice erzeugt für jede
Zahlung einen Ephemeralschlüssel; Bob erkennt die separat zugestellte
Wallet-Nachricht und **muss** ihn auf einen neuen eigenen Schlüssel übertragen.
Erst die bestätigte Übertragung ist der Abschluss des semi-interaktiven
Transfers. Alice behält bewusst ebenfalls die Spend-Fähigkeit, bis Bob diesen
Schritt bestätigt hat. Die Bibliothek speichert weder Daten noch verwendet sie
Netzwerk oder Systemzeit.

## Ausführen

```sh
cargo fmt --check
cargo test
```

## Lokaler Epic-Core- und Chain-Roundtrip

Der Fork unter `upstream/epic` implementiert zusätzlich einen klar begrenzten
Receipt-Claim-Forschungsweg. Alice erstellt einen speziellen Output mit einem
gemeinsam ableitbaren Blind; Bob scannt dessen Hint, erkennt den Betrag und
überträgt ihn mit einem `BobClaim`-Kernel auf einen regulären Zieloutput.

```sh
cd upstream/epic
cargo test -p epic_core --lib
cargo test -p epic_chain --test receipt_claim_roundtrip
cargo run -p epic_core --example receipt_stealth_demo
```

Der Chain-Test mined ausschließlich eine temporäre lokale Usernet-Testchain:
Receipt-Funding in einem Block, Bob-Claim im Folgeblock, anschließende
TxHashset-Validierung. Er startet weder P2P noch Wallet, Mempool oder Miner.

Die Public API besteht aus `ReceiverKey`, `OneSidedPayment::create`,
`ReceiverKey::scan` und `OneSidedLedger`. Die Bezeichnung `PendingEnvelope`
ist ein technischer Name für die Wallet-Nachricht, nicht ein behaupteter
Konsensstatus.

## Sicherheitsgrenzen

Das POC unter `src/` ist **keine Epic-Cash-Wallet, keine Blockchain und nicht
konsenskompatibel**. Es gibt keine realen Schlüssel, Gelder oder Netzwerke.
Der `commitment` ist ein echtes Pedersen Commitment, bietet jedoch im isolierten
Modell weder Betragsvertraulichkeit im Kontext einer echten Chain noch
Konsenssicherheit.

Der POC leitet eine gemeinsame Spend-Fähigkeit per ECDH ab und nutzt Pedersen
Commitments und AgSig nur als isolierte Primitiven. Das `PendingEnvelope` ist
**kein** Epic-Output, kein Slate, Kernel oder Range Proof und wird auch nicht
in die vorhandene 20-Byte-Range-Proof-Nachricht gepackt.

Für den Fork-Pfad existieren lokale Range-Proofs, Kernel-Signaturen, eine
vollständig validierte Transaktion und ein Block-/TxHashset-Roundtrip. Es
fehlen weiterhin P2P-Versionierung, Mempool-Admission, Reorg-Tests,
Wallet-Schlüsselverwaltung und -Recovery, laufende Node-/Miner-Integration
sowie eine formale Sicherheitsanalyse und unabhängige Audits. Die Tests
belegen ausschließlich den beschriebenen lokalen Forschungsumfang. Jede
produktive Verwendung erforderte eine finale Spezifikation, ein aktiviertes
Konsensdesign sowie unabhängige kryptografische und Implementierungs-Audits.

Weitere Details: [One-sided-POC](docs/one-sided-poc.md), [Receipt-Claim-Entwurf](docs/receipt-claim-design.md), [Kryptografische Grundlage](docs/cryptographic-foundation.md), [Protokollspezifikation](docs/protocol-spec.md), [Bedrohungsmodell](docs/threat-model.md), [Annahmen](docs/assumptions-and-open-questions.md), [Testplan](docs/test-plan.md) und die [Übergabe zur Weiterarbeit](docs/continuation-guide.md).

Die geplante lokale Chain-Validierung ist in der
[Usernet-Experimentanleitung](docs/usernet-experiment.md) dokumentiert.
