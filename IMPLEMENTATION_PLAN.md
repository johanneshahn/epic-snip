# Implementierungsplan — überholt durch PROTOCOL.md

**Status: HISTORISCH.** Dieser Plan beschreibt die
In-Memory-POC-Phase des Projekts. Er ist **abgeschlossen und
überholt durch die Funktionalspezifikation in
[PROTOCOL.md](PROTOCOL.md)**.

Die oberste Priorität dieses Projekts ist heute die
vollständige Demonstration des 3-Zahlungs-Flows
(Descriptor → 3 unabhängige Zahlungen → 3 Confirmations →
Bob scannt → Bob claimt → End-Outputs nur von Bob
auszahlbar), nicht die hier dokumentierte In-Memory-Ledger-
Simulation.

---

# Historischer Plan: In-Memory-POC

## Ziel

Erstelle im bestehenden Projektordner einen nachvollziehbaren
kryptografischen Forschungsprototypen für ein hypothetisches
Epic-Cash-SNIP-Protokoll:

- Alice erzeugt eine Zahlung ohne Bobs Online-Beteiligung.
- Bob kann die Zahlung jederzeit claimen.
- Alice kann erst ab einer definierten Blockhöhe einen
  Refund ausführen.
- Genau einer der beiden Pfade kann erfolgreich sein.

Wichtig: Dies ist ausschließlich eine lokale Modellierung und
Testumgebung. Sie ist keine Epic-Cash-Wallet, keine
Blockchain-Implementierung, nicht konsenskompatibel und darf
keine echten Schlüssel, Gelder oder Netzwerke verwenden.

## Technische Entscheidungen

- Sprache: Rust
- Build/Test: Cargo
- Modell: deterministische In-Memory-Ledger-Simulation
- Kryptografie: Schlüssel- und Signatur-Wrapper aus einer
  etablierten Rust-Bibliothek; keine selbst entwickelte
  Kryptografie
- Zeit: ausschließlich simulierte Blockhöhen, keine
  Systemzeit
- Persistenz: keine; Testdaten nur im Arbeitsspeicher
- Netzwerk: keines

## Verzeichnis

| Pfad | Inhalt |
| --- | --- |
| `src/one_sided.rs` | Kerner der POC-Modellierung |
| `tests/` | POC-Integrationstests |
| `upstream/epic` | Lokaler Fork für die Chain-Phase (abgeschlossen) |
| `docs/` | Forschungsdokumentation |

## Phasen (alle abgeschlossen)

1. Modell-Design und Zustandsautomat.
2. Krypto-Bausteine (ECDH, Pedersen, AgSig).
3. `OneSidedLedger` mit Claim/Refund-Entscheidung.
4. POC-Integrationstests.
5. Übergabe an die Chain-Phase (`upstream/epic`).
6. AliceReclaim-Entfernung und Spend-Attribution-Forschung.
7. **[Neu, ACTIVE]** Vollständige 3-Zahlungs-Demonstration
   gemäß [PROTOCOL.md](PROTOCOL.md).
