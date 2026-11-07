# Receipt-Claim & SNIP: Einseitige Wiederverwendbare Zahlungen

Status: experimentelle, lokale Usernet-Implementierung.
Weder konsenskompatibel noch kryptografisch bewiesen;
nicht für Mainnet, Floonet oder echte Gelder geeignet.

> **Oberste Priorität.** Alle Inhalte dieses Dokuments
> dienen dem in [PROTOCOL.md](../PROTOCOL.md) §2
> und §5 definierten Transaktionsfluss:
>
> 1. Bob veröffentlicht einen **Payment
>    Descriptor** einmalig.
> 2. Alice erstellt daraus **drei unabhängige
>    Zahlungen** (ohne Bob, ohne Kommunikation).
> 3. Alle drei Zahlungen werden in der **lokalen
>    Usernet-Chain** bestätigt.
> 4. Bob **detektiert** alle drei **durch Scannen
>    der Chain** (ohne Kommunikation mit Alice) und
>    **bestimmt Beträge**.
> 5. Bob **claimt** jede der drei Zahlungen in einer
>    **eigenen Transaktion**.
> 6. Alle drei Claims werden in der Usernet-Chain
>    bestätigt.
> 7. Die **End-Outputs** sind **ausschließlich von
>    Bob auszahlbar**.
>
> Gemäß [PROTOCOL.md §6](../PROTOCOL.md): Ein
> erfolgreicher isolierter Krypto- oder Konsens-Test
> **ersetzt nicht** die vollständige
> 3-Zahlungs-Demonstration.

## Ziel

Der von Alice erzeugte gemeinsame Output darf nur über
einen nachweisbaren Auflösungspfad ausgegeben werden.
Bob muss ihn auf einen eigenen Schlüssel übertragen,
damit der Transfer abgeschlossen ist. Bob darf diese
Übertragung nicht durchführen können, ohne gleichzeitig
einen für Alice nachweisbaren Claim zu hinterlassen.

**Alice hat einen konkurrierenden Reclaim-Pfad.**
Solange Bob den Output nicht in seine eigenen
Outputs übertragen hat, kann Alice ihn selbst
beanspruchen. Der erste bestätigte gültige Spend
gewinnt (First-Confirmed-Spend-Wins). Der
Alice-Reclaim-Pfad nutzt die bestehende
Standard-Mechanik von Epic Cash (gewöhnlicher
`Plain`-Kernel) und muss nicht neu erfunden werden
(PROTOCOL.md §3.9).

**Alice kann den Claim verifizieren.** Aus der
Chain kann Alice nachvollziehen, dass Bob die
Zahlungen übernommen hat (PROTOCOL.md §3.8).

Zugleich soll die Chain nicht offenlegen, welcher
spätere Zieloutput Bob gehört. Ein Außenstehender
soll daher nur erkennen können, dass ein spezieller
Output geclaimt wurde; nicht, welcher neue Output
der endgültige Bob-Output ist.

**Neu gegenüber der früheren POC-Spezifikation:**
Bob muss die Zahlungen **aus der Chain** erkennen
können — nicht aus einer externen Wallet-Nachricht.
Er muss **Beträge** aus der Chain bestimmen können.
Er muss **drei unabhängige Zahlungen** parallel
detektieren und claimen können. Die End-Outputs
müssen **ausschließlich von Bob** auszahlbar sein.
**Alice kann den Claim verifizieren** und hat einen
**konkurrierenden Reclaim-Pfad** (PROTOCOL.md
§3.8 und §3.9).

## Rollen und einmaliges Setup

Bob erzeugt einmalig ein langfristiges
Schlüsselmaterial und veröffentlicht den
öffentlichen **Payment Descriptor**. Der Descriptor
ist wiederverwendbar: aus ihm kann Alice beliebig
viele Zahlungen ableiten, ohne Bob erneut
einzubeziehen.

Der Descriptor enthält:

```text
PaymentDescriptor:
  version        : u8
  descriptor_id  : [u8; 32]   // Domain-Separator + Versions-Hash
  scan_public    : PublicKey  // sG, s = scan_secret
  receipt_spend_public : PublicKey  // pG, p = receipt_spend_secret
```

Bobs private Schlüssel (`scan_secret`,
`receipt_spend_secret`) verbleiben ausschließlich
in Bobs Wallet. Alice speichert den Descriptor;
sie benötigt pro Zahlung keine neue Antwort von
Bob.

**Achtung:** Der Descriptor ist **öffentlich**.
Jede Partei kann Zahlungen aus ihm ableiten.
Nur Bob kann die Zahlungen **erkennen** (via
`scan_secret`) und **claimen** (via
`receipt_spend_secret`).

## Stealth-Receipt-Ableitung

Die lokale Forschungsimplementierung verwendet zwei
voneinander domain-separierte ECDH-basierte
Ableitungen. Sie ist kein kryptografischer Beweis
und keine endgültige Protokollspezifikation; die
folgende Notation beschreibt den implementierten
Forschungsansatz:

```text
Bob-Descriptor: ScanPublic S = sG, ReceiptSpendPublic P = pG
Alice pro Zahlung i:
  r_i zufällig, Hint R_i = r_iG
  gemeinsamer Wert: h_i = H(domain || ECDH(r_i, S) || R_i || descriptor_id)
  Bob-Receipt-Public-Key: B_i = P + h_i * G
  Bob-Receipt-Private-Key: b_i = p + h_i   (nur Bob kennt)

Shared-Spend-Blind:
  q_i = HashToScalar(
    separate_domain || ECDH(r_i, S) || R_i || S
  )
```

Bob scannt `R_i`, berechnet `ECDH(s, R_i)` und
damit `b_i`; Alice kann zwar `B_i`, nicht aber
`b_i` berechnen. Die gleiche ECDH-Ausgabe lässt
beide Seiten `q_i` ableiten: Alice kennt `r_i`,
Bob kennt `s`. Daraus können beide dieselbe
Commitment-Blindierung für den gemeinsamen
Spezial-Output bilden. Die Ableitung für `q_i` ist
explizit von der für `h_i` getrennt, damit ein
Receipt-Autorisierungsschlüssel nicht als
Spend-Blind wiederverwendet wird.

`R_i` liegt in der lokalen Implementierung als
kanonisch serialisierter 33-Byte-Stealth-Hint am
Receipt-Output vor. Der Core baut im lokalen
Forschungs-API einen Receipt-Output mit `q_i`,
rewound dessen Betrag für Bob und erstellt daraus
eine vollständig durch
`Transaction::validate(...)` prüfbare
BobClaim-Transaktion.

## Konsensformat (Usernet-only)

Ein Receipt-Output bindet bei seiner Entstehung:

```text
SharedReceiptKeyHashes:
  bob   : [u8; 32]   // H(domain || B_i)
  alice : [u8; 32]   // H(domain || A_i)  // für Alice-Reclaim
  hint  : [u8; 33]   // R_i (kanonisch serialisiert)
```

**Wichtig:** Der `alice`-Hash ist für den
Alice-Reclaim-Pfad erforderlich. Beide Hashes
werden beim Output-Erstellungszeitpunkt fest
gelegt und sind Teil der UTXO-Identität.

Ein `BobClaim`-Kernel enthält:

```text
BobClaim:
  original_commitment = C
  receipt_public_key = B_i
  receipt_signature = Sign_{b_i}(
    domain || path || C || kernel_excess || fee
  )
```

Der Alice-Reclaim-Pfad nutzt den
**gewöhnlichen `Plain`-Kernel** aus Epic Cash.
Es gibt keinen separaten
`AliceReclaim`-Kerneltyp. Alice baut eine
Standard-Epic-Transaktion, die den
Shared-Output (`Receipt`-Feature) ausgibt.

Die Validierung im Node prüft:

1. `C` ist ein als Input dieser Transaktion
   ausgegebener Receipt-Output.
2. Für `BobClaim`: Der dort gespeicherte
   Key-Hash entspricht dem Bob-Key
   (`hashes.bob == H(B_i)`) und die
   `receipt_signature` über den kanonisch
   definierten Bindungswert ist gültig.
3. Alle normalen Regeln für
   Transaktionsbilanz, Range Proofs, Fee und
   Kernel-Signatur erfüllen sich ebenfalls.
4. Jede Ausgabe eines Receipt-Outputs enthält
   genau einen passenden Resolution-Kernel
   (entweder `BobClaim` oder `Plain`).
   *(Anmerkung: Die aktuelle Konsensregel
   verlangt BobClaim. Eine Anpassung ist
   erforderlich, um §3.9 zu erfüllen — siehe
   CURRENT_STEP.md.)*
5. **First-Confirmed-Spend-Wins:** Wenn Bob
   bereits claimed hat (Output ist consumed),
   wird ein späterer Alice-Spend abgewiesen
   (`AlreadySpent`). Umgekehrt gilt dasselbe.

## Scan für Bob (neuer, noch zu implementierender Teil)

**Diese Api ist noch nicht in
`upstream/epic` implementiert.** Sie ist der
wichtigste Baustein, der für die SNIP-Akzeptanz
fehlt.

```rust
// Vorschlag für die Scan-Api
pub fn scan_for_bob(
    receipt_spend_secret_key: SecretKey,
    scan_secret_key: SecretKey,
    descriptor: &PaymentDescriptor,
    chain: &ChainHandle,
) -> Vec<DetectedPayment>
{
    // Für jeden Receipt-Output in der Chain:
    //   1. ECDH(scan_secret, hint) → h_i
    //   2. B_i = P + h_i * G
    //   3. H(B_i) == hashes.bob? → Detektion!
    //   4. q_i ableiten → Betrag rewinden
    // → DetectedPayment { output_index, amount, hint }
}
```

Jeder `DetectedPayment` enthält genug
Information, damit Bob seine Claim-Transaktion
bauen kann, ohne Alice zu kontaktieren.

**Effizienz-Frage (offen):** Der naive Scan prüft
jeden `Receipt`-Output gegen jeden
`descriptor_id`. Für die Usernet-Akzeptanz
genügt das; für ein produktives Wallet wäre ein
Index über `H(descriptor_id)` in
`SharedReceiptKeyHashes` erforderlich.

## Vorgeschlagener 3-Zahlungs-Flow (Akzeptanz)

```text
Bob:
  d = PaymentDescriptor.create()
  // veröffentlicht d an Alice (einmalig)

Alice:
  payment_1 = Payment.create(d, amount=100, alice_blind_1)
  payment_2 = Payment.create(d, amount=200, alice_blind_2)
  payment_3 = Payment.create(d, amount=300, alice_blind_3)
  // bringt alle drei in Usernet-Blöcke:
  mine_block(payment_1)
  mine_block(payment_2)
  mine_block(payment_3)

Bob (ohne Kommunikation mit Alice):
  detected = scan_for_bob(
    receipt_spend_secret, scan_secret, d, chain
  )
  // detected.length == 3
  // detected[0].amount == 100
  // detected[1].amount == 200
  // detected[2].amount == 300

  for payment in detected:
    claim_tx = Claim.build(
      receipt_spend_secret, scan_secret, d, payment
    )
    mine_block(claim_tx)
  // alle drei Claims bestätigt

Alice (Verifizierung, PROTOCOL.md §3.8):
  for payment in [payment_1, payment_2, payment_3]:
    verify_bob_claimed(payment)
    // Alice kann aus der Chain nachvollziehen,
    // dass Bob den Output geclaimt hat und die
    // Mittel nun in Bobs End-Outputs stecken.

End-Outputs:
  // jeder End-Output ist nur von Bobs
  // Ziel-Schlüssel auszahlbar; jeder
  // andere Spend-Versuch wird abgewiesen.

Alternativ: Alice-Reclaim (PROTOCOL.md §3.9):
  // Falls Bob NICHT claimt:
  // Alice baut eine gewöhnliche Epic-Transaktion
  // (Plain-Kernel), die den Shared-Output ausgibt.
  alice_spend = PlainSpend.build(
    shared_blind, destination_blind
  )
  mine_block(alice_spend)
  // Alice-Spend erfolgreich, Zahlung geht
  // an Alices eigenen Output.
  // Bob kann den Output danach NICHT mehr
  // claimen (bereits spent).
```

## Lokaler Implementierungsstand

Im lokalen Subprojekt `upstream/epic` ist ein
erster Usernet-only Core-Schritt implementiert und
durch Core-Unit-Tests abgedeckt:

- `OutputFeatures::Receipt` trägt einen
  Bob-only-`SharedReceiptKeyHashes`
  (`bob: [u8; 32]`) sowie einen kanonisch
  geprüften 33-Byte-Stealth-Hint. Input, Output
  und MMR-Output-Identifier serialisieren diese
  Daten mit.
- Der Core enthält ein experimentelles
  ECDH-/Tweak-Primitiv: Alice kann aus Bobs
  öffentlichem Descriptor und einem frischen
  Ephemeralschlüssel einen Bob-Receipt-Public-Key
  bilden, während Bob aus Hint und seinen
  privaten Descriptor-Schlüsseln den zugehörigen
  privaten Receipt-Key ableitet.
- Der Core leitet außerdem einen getrennten
  gemeinsamen Spend-Blind ab. Alice und Bob
  erhalten daraus unabhängig dasselbe
  Commitment für den Spezial-Output. Der
  Forschungs-Builder erstellt daraus einen
  gültigen Receipt-Range-Proof; Bob kann dessen
  Betrag mit einem aus dem Shared-Blind
  abgeleiteten Rewind-Nonce lesen.
- Ein lokaler Roundtrip baut anschließend eine
  ein-input/ein-output BobClaim-Transaktion mit
  normalem Zieloutput, gültigem Range Proof,
  Kernel-Signatur, Receipt-Signatur und
  korrekter Kernel-Summe. Der Test ruft
  `Transaction::validate(...)` im Usernet-Modus
  auf.
- `libtx::build::receipt_output(...)` erzeugt
  einen experimentellen Receipt-Output mit
  Key-Hashes und Hint; der reguläre
  Proof-Builder erzeugt weiterhin dessen Range
  Proof.
- `KernelFeatures::BobClaim` (U8 = 3) trägt die
  jeweilige Fee und ein `ReceiptClaim`-Transkript.
- Das Transkript bindet per domain-separierter
  Signatur Pfad, ursprüngliches Commitment,
  Kernel-Excess und Fee. Der Node prüft die
  normale Kernel-Signatur und anschließend diese
  Bindung.
- Eine Transaktions-Body-Prüfung verlangt eine
  Eins-zu-eins-Zuordnung zwischen jedem
  ausgegebenen Receipt-Input und einem passenden
  Resolution-Kernel samt registriertem
  Receipt-Key-Hash.
- Receipt-Features werden in dieser
  experimentellen Stufe außerhalb des
  Usernet-Modus abgelehnt.
- Der Alice-Reclaim-Pfad nutzt den
  **gewöhnlichen `Plain`-Kernel** aus Epic Cash
  (PROTOCOL.md §3.9). Es gibt keinen separaten
  `AliceReclaim`-Kerneltyp.

Ein lokaler Chain-Integrationstest
(`receipt_claim_roundtrip.rs`) erzeugt einen
Usernet-Genesisblock, mint eine reife
Test-Coinbase, nimmt Alice' Receipt-Funding-
Transaktion in einen Block auf und nimmt Bobs
Claim in einem späteren Block auf. Er prüft
danach, dass der Receipt-UTXO verbraucht, Bobs
Zieloutput unspent und die vollständige
TxHashset-Validierung erfolgreich ist.

**Noch nicht vorhanden:**

- Die `scan_for_bob`-Api (s.o.)
- Ein Integrationstest, der drei unabhängige
  Zahlungen in einem Durchlauf demonstriert
- Die `PaymentDescriptor`-Abstraktion als
  eigener Typ (aktuell implizit über
  `receipt_spend_public_key` +
  `scan_public_key`)
- Wallet-Integration von Descriptor-Erzeugung,
  Scan und Claim
- P2P-Versionierung, Mempool-Admission,
  laufende Node-/Miner-Integration

## Was öffentlich und privat bleibt

| Information | Sichtbarkeit |
| --- | --- |
| Payment Descriptor | dauerhaft öffentlich (von Bob veröffentlicht) |
| Stealth-Hint `R_i` pro Zahlung | öffentlich, aber pro Zahlung frisch; ohne Bobs Scan-Key keine öffentliche Zuordnung zu `B_i` |
| Spezial-Output `C` wurde geclaimt | dauerhaft öffentlich über den Receipt-Claim-Kernel |
| Ein einmaliger Receipt-Key hat aufgelöst | dauerhaft öffentlich |
| Bobs endgültiger Zieloutput | nicht Bestandteil des Receipt-Claim-Kernels; nur Bob erkennt ihn beim Wallet-Scan |
| Betrag | weiterhin durch Commitment und Range Proof verborgen, sofern keine Partei ihn offenlegt |
| Bobs langfristiges Schlüsselmaterial | **nie** öffentlich |

## Atomare Eigenschaft

Ein Bob-Claim ist nur gültig, wenn seine
Receipt-Signatur im gültigen `BobClaim`-Kernel
enthalten ist. Der Alice-Reclaim nutzt den
gewöhnlichen `Plain`-Kernel und benötigt
keine zusätzliche Receipt-Signatur.

**First-Confirmed-Spend-Wins.** Wenn Bob den
Output bereits geclaimt hat (Output ist
consumed), wird ein späterer Alice-Spend
abgewiesen (`AlreadySpent`). Umgekehrt gilt
dasselbe: Wenn Alice den Output ausgegeben
hat, kann Bob den Output nicht mehr claimen.

**Alice kann den Claim verifizieren.** Aus der
Chain kann Alice nachvollziehen, dass Bob den
Output geclaimt hat, indem sie den
`BobClaim`-Kernel findet und die
Signaturen und Key-Hashes prüft
(PROTOCOL.md §3.8).

## Offene Fragen

1. **Scan-Effizienz.** Wie wird der Chain-Scan
   für Bobs Wallet skaliert? (Für die
   Usernet-Akzeptanz ist der naive Scan
   ausreichend.)
2. **Descriptor-Index.** Soll
   `SharedReceiptKeyHashes` einen separaten
   Index über `H(descriptor_id)` enthalten,
   damit der Scan nicht jeden
   `Receipt`-Output gegen jeden
   `descriptor_id` prüfen muss?
3. **Multi-Zahlungs-Korrelierbarkeit.** Können
   drei Zahlungen aus demselben Descriptor
   öffentlich korreliert werden?
   (PROTOCOL.md §3.7.)
4. **Betrag-Leakage beim Scan.** Findet der
   Scan zusätzlich zum Output selbst
   Betragsinformation, die über den
   Range-Proof hinausgeht?
5. **Claim-Order.** Kann Bob mehrere Claims in
   einer Transaktion bündeln, oder muss jeder
   Claim eine eigene Transaktion sein?
   (PROTOCOL.md §2.5 legt „eigene
   Transaktion" nahe.)
6. **End-Output-Exklusivität.** Wie wird die
   Exklusivität der End-Outputs für Bob
   kryptografisch nachgewiesen (PROTOCOL.md
   §3.4)?
7. **Reorg-Behandlung.** Was passiert mit
   Bobs Scan-Ergebnissen, wenn ein Block
   reorged wird?
8. **Wallet-Integration.** Wie werden
   Descriptor-Erzeugung, Scan und Claim in
   die bestehende Epic-Wallet integriert,
   ohne die Mainnet-Kompatibilität zu
   gefährden?
9. **Alice-Reclaim-Konsensanpassung.** Wie
   wird der Konsens so angepasst, dass ein
   `Plain`-Kernel einen `Receipt`-Input ausgeben
   darf, während Bob-Claims weiterhin die
   `ReceiptClaim`-Attribution erfordern? (Siehe
   CURRENT_STEP.md, Optionen A und B.)
10. **Alice-Verifizierung.** Wie genau kann
    Alice aus der Chain verifizieren, dass
    Bob die Zahlungen übernommen hat?
    (PROTOCOL.md §3.8.)
11. **Race-Condition.** Was passiert, wenn
    Alice und Bob gleichzeitig Claim/Reclaim
    versuchen? Wie wird der Gewinner
    determiniert?

Diese Fragen werden im nächsten Design-Pass
behandelt. Bis dahin gilt PROTOCOL.md als
oberste Priorität.
