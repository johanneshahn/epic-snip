# One-sided shared-redeemable: implementierter Forschungs-POC

## Zweck und Reichweite

`src/one_sided.rs` modelliert einen Ablauf ohne Antwort von Bob bei der
Zahlungserstellung. Er verfolgt absichtlich **keinen** on-chain
„Pending“-Status und keinen Refund-Pfad. Der ursprüngliche Output ist für Bob
automatisch erkennbar und von Alice und Bob gemeinsam ausgebbar:

1. Bob erzeugt `ReceiverKey` und veröffentlicht nur dessen öffentlichen
   Scan-Schlüssel.
2. Alice erzeugt für **jede** Zahlung einen neuen Ephemeralschlüssel `a` und
   leitet aus `ECDH(a, B)` einen domänenseparierten gemeinsamen Blind ab.
3. Aus diesem Blind entsteht ein Pedersen Commitment sowie ein
   `PendingEnvelope` mit dem ephemeren öffentlichen Schlüssel `A`.
4. Bob erhält das Envelope über einen modellierten einseitigen
   Wallet-Nachrichtenkanal, scannt es mit `b`, berechnet `ECDH(b, A)` und
   erkennt nur bei passendem Commitment seine Zahlung.
5. Bob **muss** den erkannten Output auf ein neues Commitment mit einem nur ihm
   bekannten Schlüssel übertragen. Erst nach Bestätigung dieser Übertragung ist
   der semi-interaktive Transfer abgeschlossen und Bob kontrolliert den Betrag
   exklusiv. Alice kann vorher mit derselben gemeinsamen Fähigkeit ausgeben.
   Das ist keine Schwachstelle des angestrebten Modells, sondern ausdrücklich
   akzeptiert: Der erste gültige bzw. bestätigte Spend gewinnt.

Die Bezeichnung `PendingEnvelope` im Code benennt nur die externe
Wallet-Nachricht. Sie darf nicht als durch den Epic-Konsens erzwingbarer Status
verstanden werden.

## Was der Code tatsächlich prüft

- ECDH liefert auf Alice- und Bob-Seite denselben abgeleiteten Blind.
- Ein anderer Scan-Schlüssel erkennt das Envelope nicht.
- Bobs verpflichtende Abschlussübertragung wechselt im lokalen Ledger von
  `Pending` zu `Claimed`.
- Ein vorheriger Alice-Übergang setzt `SenderSpent`; eine spätere Bob-Ausgabe
  wird danach abgewiesen. Dieser Test dokumentiert die bewusst akzeptierte
  First-Spend-Wins-Semantik.
- Die AgSig-Autorisierung bindet Zahlungs-ID, Ursprung- und Ziel-Commitment,
  Betrag und Pfad aneinander.

## Ausdrückliche Nicht-Eigenschaften

Dies ist weder ein Sicherheitsbeweis noch ein Epic-Cash-Protokoll. Insbesondere:

- `PendingEnvelope` ist eine externe Wallet-Nachricht. Das aktuelle Epic-Output
  und die bestehende Range-Proof-Nachricht tragen den benötigten 33-Byte-
  Ephemeralschlüssel nicht nachweislich.
- Der `OneSidedLedger` baut keine Transaktion, keinen Kernel und keinen Range
  Proof. Er erzwingt weder eine Mimblewimble-Bilanzgleichung noch UTXO-,
  Mempool- oder Reorg-Regeln.
- Der Zielschlüssel wird nur im Modell als Bob-eigenes Output-Commitment
  dargestellt. Es folgt daraus keine on-chain erzwingbare Eigentums- oder
  Exklusivitätsaussage.
- Alice kann den gemeinsamen Blind ebenfalls ableiten und daher vor Bobs
  verpflichtender Abschlussübertragung konkurrierend ausgeben. Bob hat bis zu
  deren Bestätigung keine Verfügbarkeitsgarantie. Beide Eigenschaften sind für
  diesen POC akzeptierte Designentscheidungen, keine zu beseitigenden
  Anforderungen.
- Ein Payment Proof kann nach einer echten finalen Transaktion als
  prüfbarer Zahlungsbeleg dienen. Er verleiht aber weder Erkennungs- noch
  Spend-Rechte für den ursprünglichen Output und ändert diese Semantik nicht.

## Geplanter Receipt-Claim und Privatsphäre

Die gewünschte nicht abstreitbare Bestätigung wird nicht durch den bestehenden
POC erbracht. Der weiterführende Entwurf in
[`receipt-claim-design.md`](receipt-claim-design.md) sieht vor, dass Bob beim
verpflichtenden Abschluss-Claim einen einmaligen Receipt-Key signiert. Der
Konsensnachweis soll den ursprünglichen Spezial-Output und Bobs Claim, **nicht
aber den späteren Bob-Zieloutput**, verbinden. Somit kann Alice den Claim
beweisen, während Außenstehende Bobs neue Wallet-Ausgabe nicht zuordnen können.
- Schlüsselrotation, Wiederherstellung, Nachrichtenverfügbarkeit,
  Metadaten-Privatsphäre, DoS, Front-running und Descriptor-Wiederverwendung
  sind nicht bewertet.

Die Tests unter `tests/one_sided_pending.rs` sind deshalb ausschließlich
regression tests für dieses lokale Modell.
