# Epic SNIP — Einseitige Wiederverwendbare Zahlungen

> **Experimenteller Forschungsstand — nicht für reale Gelder.** Der lokale
> `upstream/epic`-Fork enthält einen Usernet-only Konsensentwurf und ist weder
> mit Epic Mainnet/Floonet kompatibel noch kryptografisch auditiert. Er darf
> nicht an EpicCash-Upstreams gepusht und nicht als produktionsreif verstanden
> werden.

## Oberste Priorität

Dieses Projekt verfolgt **eine** Sache: die vollständige Demonstration des
in [PROTOCOL.md](PROTOCOL.md) definierten Transaktionsflusses.

Bob veröffentlicht einen **Payment Descriptor** einmalig.
Alice erstellt daraus **drei unabhängige Zahlungen**
(ohne Bob, ohne Kommunikation) und bringt sie in der
lokalen **Usernet-Chain** zur Bestätigung.
Bob **detektiert** alle drei **durch Scannen der Chain**,
bestimmt **dass** und **den Betrag**, und **claimt** jede
in einer **eigenen Transaktion** auf einen
**ausschließlich von ihm auszahlbaren** Output.
Alle drei Claims werden in der Usernet-Chain bestätigt.

**Zusätzliche Anforderungen (PROTOCOL.md §3.8, §3.9):**
- **Alice kann aus der Chain verifizieren**, dass Bob die
  Zahlungen übernommen hat.
- **Alice kann die Shared Outputs jederzeit selbst
  beanspruchen**, solange Bob sie nicht bereits in seine
  eigenen Outputs übertragen hat. Der erste bestätigte
  gültige Spend gewinnt (First-Confirmed-Spend-Wins).

Jede andere Arbeit in diesem Repo — die In-Memory-POC unter
`src/one_sided.rs`, der Receipt-Claim-Forschungspfad unter
`upstream/epic`, die Attributionsforschung — ist
**abgeschlossen und überholt durch diese Spezifikation**.
Sie bleibt als historische Aufzeichnung erhalten.

## Ausführen

```sh
cargo fmt --check
cargo test
```

### Ziel-Test (in Arbeit)

Der zu schreibende Akzeptanz-Integrationstest muss den
vollständigen 3-Zahlungs-Flow demonstrieren:

```sh
# (noch nicht vorhanden — folgt aus PROTOCOL.md §5)
cargo test -p epic_chain --test snip_acceptance
```

### Bestehende, abgeschlossene Tests (historisch)

```sh
cd upstream/epic
cargo test -p epic_core --lib
cargo test -p epic_chain --test receipt_claim_roundtrip
cargo test -p epic_chain --test spend_attribution
cargo run -p epic_core --example receipt_stealth_demo
```

Diese Tests dienen der Recherche und der Überprüfung einzelner
Bausteine. Gemäß PROTOCOL.md §6 **ersetzen sie nicht** die
vollständige 3-Zahlungs-Demonstration.

## Sicherheitsgrenzen

Keine Mainnet-Kompatibilität. Keine echten Gelder. Keine
Netzwerke. Alle Schlüssel sind ephemere Testmaterialien.
Die lokale Range-Proof-, Kernel-Signatur- und
TxHashset-Validierung belegen ausschließlich den
beschriebenen Usernet-Forschungsumfang.

## Dokumentation

- **Oberste Priorität und Funktionalspezifikation:** [PROTOCOL.md](PROTOCOL.md)
- **Fortlaufender Status:** [CURRENT_STEP.md](CURRENT_STEP.md)
- Historisch: [One-sided-POC](docs/one-sided-poc.md), [Receipt-Claim-Entwurf](docs/receipt-claim-design.md), [Kryptografische Grundlage](docs/cryptographic-foundation.md), [Protokollspezifikation (POC)](docs/protocol-spec.md), [Bedrohungsmodell](docs/threat-model.md), [Annahmen](docs/assumptions-and-open-questions.md), [Testplan (POC)](docs/test-plan.md), [Übergabe zur Weiterarbeit](docs/continuation-guide.md), [Usernet-Experiment](docs/usernet-experiment.md), [Spend-Attribution-Forschung](docs/SNIP_SPEND_ATTRIBUTION_RESEARCH.md)

Alle Dokumentations-Links auf die historischen Pfade
gilt: sie beschreiben den **abgeschlossenen**
Forschungsstand, **nicht** den in PROTOCOL.md
definierten Zielzustand.
