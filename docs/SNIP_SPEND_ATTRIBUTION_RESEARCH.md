# SNIP: Spend Attribution — Research Report

**Status: HISTORISCH — abgeschlossen und überholt durch
[PROTOCOL.md](../PROTOCOL.md).**

Dieser Report dokumentiert die
Spend-Attributionsforschung aus der Phase, in der
es noch eine `AliceReclaim`-Mechanik gab. Die
`AliceReclaim`-Mechanik wurde **vollständig
entfernt**; der Report bleibt als historische
Aufzeichnung erhalten.

**Die oberste Priorität dieses Projekts ist heute
die vollständige 3-Zahlungs-Demonstration gemäß
[PROTOCOL.md](../PROTOCOL.md) §5.** Alle
folgenden Inhalte sind **historisch** und keine
Anleitung für weitere Entwicklung.

---

## Zusammenfassung (historisch)

**Alice kann kryptografisch verifizieren, dass eine
bestätigte Transaktion ein gültiger BobClaim ist**
(d.h., Bob hat die Ausgabe autorisiert),
ausschließlich mit bestätigten Chain-Daten.

**Alice kann den gemeinsamen Receipt-Output nicht
selbst ausgeben.** Die Konsensvalidierung verlangt,
dass jeder Input mit `features = Receipt`
eins-zu-eins mit einem
Receipt-Auflösungs-Kernel (derzeit
`BobClaim`) übereinstimmt. Ein
gewöhnlicher Kernel, der einen Receipt-Output
ausgibt, wird mit
`InvalidReceiptClaim` abgelehnt.

Das Attributions-Modell lautet jetzt:

- **Wenn Bob claimt:** Der Kernel hat
  `features = BobClaim`, ein
  `receipt_claim` ist vorhanden, und
  `receipt_claim.receipt_public_key`
  entspricht `hashes.bob` (dem Bob-spezifischen
  Hash). Alice kann das aus öffentlichen Daten
  verifizieren.
- **Wenn jemand anderes eine Ausgabe
  versucht:** Der Konsens lehnt die
  Transaktion ab. Es gibt keinen
  „Alice-Spend"-Pfad.

## Q1: Kann Alice einen Bob-Claim von jeder
anderen Ausgabe unterscheiden?

**Ja, für bestätigte Transaktionen.** Wenn Alice
eine bestätigte Transaktion beobachtet, die ihren
gemeinsamen Receipt-Output ausgibt:

- Wenn `kernel.features` `BobClaim { fee }` und
  `kernel.receipt_claim` vorhanden ist, prüft
  sie `receipt_claim.receipt_public_key`
  gegen `hashes.bob`. Bei Übereinstimmung
  wurde der Claim kryptografisch von Bob
  autorisiert (nur Bob besitzt den
  `receipt_spend_secret_key`, der zur
  Ableitung des `bob_receipt_key` benötigt
  wird).
- Jeder andere Kernel-Typ (Plain usw.) wäre
  vom Konsens abgelehnt worden und kann
  daher nicht in einem bestätigten Block
  erscheinen.

**Es existiert kein anderer Ausgabepfad.** Der
gemeinsame Receipt-Output kann nur durch einen
BobClaim konsumiert werden. Es gibt keine
Alice-Reclaim, keine gewöhnliche Ausgabe, keinen
alternativen Auflösungspfad.

## Q2: Welche kryptografische Bindung
verbindet den BobClaim spezifisch mit Bob?

Die Bindungskette lautet:

1. **Alice erstellt den Receipt** mit
   `SharedReceiptKeyHashes { bob: H(bob_receipt_key) }`.
2. **Bob leitet `bob_receipt_key` ab**
   mit `receipt_spend_secret_key` (den nur Bob
   kennt):
   `bob_receipt_key = receipt_spend_secret_key + tweak`
   (wobei der Tweak aus Alices
   Ephemeralschlüssel abgeleitet wird).
3. **Bobs Claim** enthält
   `receipt_public_key = bob_receipt_key * G`.
4. **Der Konsens validiert**
   (in `verify_receipt_claim`):
   `hashes.bob == receipt_key_hash(claim.receipt_public_key)`.

Die entscheidende Eigenschaft: **Nur Bob kann
einen `receipt_public_key` erzeugen, dessen
Hash `hashes.bob` entspricht**, da nur Bob den
`receipt_spend_secret_key` kennt. Alice kann
dies ohne Bobs Geheimes nicht fälschen.

Zusätzlich verlangt die AgSig in
`ReceiptClaim::verify()` den privaten
`bob_receipt_key`, um
`(path, original_commitment, kernel_excess, fee)`
zu signieren. Selbst wenn Alice
`bob_receipt_public_key` wüsste, könnte sie
keine gültige Signatur erzeugen.

## Q3: Kann Alice einen BobClaim fälschen
(Bob imitieren)?

**Nein.** Alice müsste einen
`receipt_public_key` erzeugen, sodass
`H(receipt_public_key) == hashes.bob` ist. Da
`hashes.bob = H(bob_receipt_key)` und
`bob_receipt_key` aus
`receipt_spend_secret_key` abgeleitet wird
(den Alice nicht kennt), kann Alice den
korrekten `receipt_public_key` nicht
berechnen.

**Wichtiger getester Fall (Experiment C,
Fall 2):** Alice *kennt*
`bob_receipt_public_key` (es ist die Ausgabe von
`derive_for_sender`). Sie kann es in den
`ReceiptClaim` einfügen und der
Hash-Check **besteht**. Sie kann jedoch
keine gültige AgSig erzeugen, da sie
`bob_receipt_key` (den privaten Schlüssel)
nicht kennt. Der Signatur-Check
**schlägt fehl**, und die Transaktion wird
abgelehnt.

**Experimentelle Verifizierung:**
- **Fall 1 (zufälliger PubKey):**
  Hash-Check schlägt fehl. Abgelehnt.
- **Fall 2 (korrekter Bob-PubKey +
  gefälschte Signatur):** Hash-Check
  besteht, AgSig schlägt fehl. Abgelehnt.
- **Fall 3 (kopierte öffentliche Felder
  aus vorherigem Claim):** Signatur an
  anderen Kernel gebunden. Abgelehnt.
- **Fall 4 (Alices eigene Geheimnisse):**
  Hash-Check schlägt fehl. Abgelehnt.
- **Fall 5 (Alice-kontrolliertes Ziel):**
  Defense-in-Depth — selbst wenn alle Checks
  bestünden, wäre das Ziel
  Alice-kontrolliert (ohne gültigen Claim
  nicht beobachtbar).

## Q4: Was passiert, wenn Bob den
`receipt_claim` aus seinem Kernel
weglässt?

Die Konsensvalidierung lehnt den Kernel ab,
weil `ReceiptClaim` ein erforderliches Feld
für `features = BobClaim` ist. Ohne
`receipt_claim` wird die Transaktion mit
`MissingReceiptClaim` abgelehnt.

## Q5: Kann ein Außenstehender erkennen,
dass ein Output von Bob claimt wurde?

**Ja, das ist gewollt.** Der
`BobClaim`-Kernel ist ein
öffentlich einsehbarer Spezial-Kernel.
Ein Außenstehender kann erkennen:
- dass ein Receipt-Output ausgab,
- dass die Ausgabe durch einen
  `BobClaim`-Kernel erfolgte,
- dass die Signatur gültig ist.

**Nein, nicht was Bob daraus
wurde.** Der Zieloutput des Claims ist ein
gewöhnlicher Output, der nicht im
`BobClaim`-Kernel referenziert wird.
Ein Außenstehender kann die Claim-Transaktion
nicht zu Bobs späteren Ausgaben
korrelieren.

## Test-Matrix

| Test | Ziel | Ergebnis |
| --- | --- | --- |
| A | Gültiger BobClaim wird akzeptiert | ✅ PASS |
| B1 | Alice-Spend abgewiesen | ✅ PASS |
| B2 | Alice-Fälschung abgewiesen | ✅ PASS |
| B3 | Bob-Spend mit Metadaten abgewiesen | ✅ PASS |
| B4 | Bob-Spend ohne Metadaten abgewiesen | ✅ PASS |
| C | 5 Fälle: Bob-Imitation abgewiesen | ✅ PASS |
| C-full | Volle Chain-Roundtrip + Klassifizierung | ✅ PASS |
| D1 | Persistenz: Receipt-Hashes über Serialisierung | ✅ PASS |
| D2 | Kernel-Struktur: BobClaim-Transkript | ✅ PASS |
| D3 | Unfälschbarkeit: Hash-Bindung | ✅ PASS |
| D4 | Unfälschbarkeit: AgSig-Bindung | ✅ PASS |
| E | Statische Analyse: keine Alice-Spend-Pfade | ✅ PASS |
| Bonus | Vollständige Attributionskette | ✅ PASS |

**Gesamt: 13/13 PASS.**

## Schlussfolgerung (historisch)

Das Spend-Attributionsmodell ist
**gültig**. Der BobClaim-Mechanismus ist
unanfällig, es gibt keinen Umgehungspfad,
und die Attributierung ist aus öffentlichen
Chain-Daten verifizierbar.

## Überführung in das neue SNIP-Protokoll

Die hier dokumentierten Eigenschaften bleiben **in Kraft**
und bilden das Fundament des neuen SNIP-Protokolls:

- BobClaim bleibt der **primäre** Ausgabepfad für
  Receipt-Outputs.
- Die AgSig-Bindung bleibt die Grundlage dafür, dass nur
  Bob Claimen kann.
- Die Hash-Bindung an `SharedReceiptKeyHashes.bob` bleibt
  die Grundlage dafür, dass Alice und Außenstehende die
  Attributierung verifizieren können.

**Änderung für das SNIP-Protokoll:**
- Der Alice-Reclaim-Pfad nutzt den
  **gewöhnlichen `Plain`-Kernel** aus Epic Cash
  (PROTOCOL.md §3.9). Es gibt keinen separaten
  `AliceReclaim`-Kerneltyp.
- Eine Konsensanpassung ist erforderlich, um
  `Plain`-Kernels für `Receipt`-Inputs zu
  erlauben (siehe CURRENT_STEP.md, Optionen A
  und B).
- First-Confirmed-Spend-Wins: Wenn Bob bereits
  claimed hat, wird ein späterer Alice-Spend
  abgewiesen (`AlreadySpent`). Umgekehrt gilt
  dasselbe.

**Neu für das SNIP-Protokoll:**
- Bob muss die Zahlungen **durch Scannen der Chain**
  erkennen (nicht durch Wallet-Nachricht).
- Bob muss **drei unabhängige Zahlungen** parallel
  detektieren und claimen können.
- Die End-Outputs müssen **ausschließlich von Bob**
  auszahlbar sein.
- Es muss eine `PaymentDescriptor`-Abstraktion geben.
- Es muss eine neue Integrationstest geben, die den
  3-Zahlungs-Flow end-to-end demonstriert.
- Alice kann aus der Chain verifizieren, dass Bob die
  Zahlungen übernommen hat (PROTOCOL.md §3.8).
- Alice kann die Shared Outputs jederzeit selbst
  beanspruchen, solange Bob sie nicht bereits übertragen
  hat (PROTOCOL.md §3.9).

Diese neuen Punkte stehen in
[CURRENT_STEP.md](../CURRENT_STEP.md) und
[PROTOCOL.md](../PROTOCOL.md).
