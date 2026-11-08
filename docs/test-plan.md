# Testplan — überholt durch PROTOCOL.md

**Status: HISTORISCH und überholt durch
[PROTOCOL.md](../PROTOCOL.md).**

Dieser Testplan beschrieb die POC-Tests in `tests/` und
deckt **nicht** die heute geforderte 3-Zahlungs-
Demonstration ab.

## Neue oberste-Priorität-Tests (aus PROTOCOL.md §5)

Der zu schreibende Akzeptanz-Integrationstest
(`upstream/epic/chain/tests/snip_acceptance.rs`) muss
**einen einzigen** Flow demonstrieren:

1. Bob erzeugt einen Payment Descriptor (einmalig).
2. Alice erzeugt 3 unabhängige Zahlungen aus dem
   Descriptor — ohne Bob, ohne Kommunikation.
3. Alle 3 Zahlungen werden in der lokalen Usernet-Chain
   bestätigt.
4. Bob scannt die Chain (ohne Kommunikation mit Alice),
   detektiert alle 3 Zahlungen und bestimmt die Beträge.
5. Bob claimt jede der 3 Zahlungen in einer eigenen
   Transaktion.
6. Alle 3 Claims werden in der Usernet-Chain bestätigt.
7. Die End-Outputs sind ausschließlich von Bob auszahlbar
   (Test: jeder andere Versuch, sie auszugeben, wird
   abgewiesen).

```sh
cargo test -p epic_chain --test snip_acceptance
```

## Bestehende, abgeschlossene Tests (historisch)

| Test | Abdeckung | Status |
| --- | --- | --- |
| `tests/one_sided_pending.rs` | In-Memory POC: Scan, Claim, SenderSpend, AgSig-Bindung | ✅ abgeschlossen |
| `upstream/epic/chain/tests/receipt_claim_roundtrip.rs` | 1 Zahlung: Funding → Claim → Bestätigung in Usernet-Chain | ✅ abgeschlossen |
| `upstream/epic/chain/tests/spend_attribution.rs` | BobClaim-Unfälschbarkeit, Attributierung, keine Alice-Spend-Pfad | ✅ abgeschlossen |
| `cargo test -p epic_core --lib` | Kern-Krypto, Serialisierung, Transaktionsvalidierung | ✅ abgeschlossen |

Diese Tests sind **historisch** und werden nicht mehr
modifiziert. Sie dienen der Recherche und als Fundament
für den neuen 3-Zahlungs-Flow.

## Was der POC-Testplan abdeckte (historisch)

Die Integrationstests prüften:

- Bob erkennt und claimt ein `OneSidedPayment` über seinen
  Scan-Schlüssel.
- Ein anderer Receiver-Schlüssel erkennt das Envelope
  nicht.
- Alice kann den Output konkurrierend ausgeben, bevor Bob
  claimt; ein späterer Bob-Claim wird dann abgewiesen
  (`AlreadySpent`).
- Die AgSig-Autorisierung ist an `payment_id` und
  Commitment gebunden.
- Pedersen Commitments haben die erwartete additive
  Eigenschaft.

Erwartet wurde bei jeder ungültigen Autorisierung ein
Fehler ohne Zustandswechsel und bei konkurrierenden
gültigen Versuchen genau ein erfolgreicher Übergang.
Nicht abgedeckt waren reale Netzwerke, Konsens, Reorgs,
Side Channels, Zufallsgüte, Persistenz, Last, DoS, formale
Verifikation und echte Mimblewimble-Kryptografie.
Erfolgreiche Tests waren kein Sicherheitsnachweis.
