# One-sided shared-redeemable POC — HISTORISCH

**Status: ABGESCHLOSSEN und überholt durch
[PROTOCOL.md](../PROTOCOL.md).**

Dieses Dokument beschreibt die in `src/one_sided.rs`
implementierte In-Memory-Modellierung. Sie ist **nicht**
die in PROTOCOL.md geforderte Ziel-Implementierung und
keine Anleitung für weitere Entwicklung. Der
vollständige 3-Zahlungs-Flow (Descriptor → 3 unabhängige
Zahlungen → 3 Confirmations → Bob scannt → Bob claimt →
End-Outputs nur von Bob auszahlbar) wird in
`upstream/epic` realisiert; dieses In-Memory-Modell
spielt in der Akzeptanz keine Rolle.

---

## Zweck und Reichweite (historisch)

`src/one_sided.rs` modelliert einen Ablauf ohne Antwort von
Bob bei der Zahlungserstellung. Er verfolgte absichtlich
**keinen** on-chain „Pending“-Status und keinen Refund-Pfad.
Der ursprüngliche Output war für Bob automatisch erkennbar
und von Alice und Bob gemeinsam ausgebbar:

1. Bob erzeugte `ReceiverKey` und veröffentlichte nur
   dessen öffentlichen Scan-Schlüssel.
2. Alice erzeugte für **jede** Zahlung einen neuen
   Ephemeralschlüssel `a` und leitete aus `ECDH(a, B)` einen
   domänenseparierten gemeinsamen Blind ab.
3. Aus diesem Blind entstand ein Pedersen Commitment sowie
   ein `PendingEnvelope` mit dem ephemeren öffentlichen
   Schlüssel `A`.
4. Bob erhielt das Envelope über einen modellierten
   einseitigen Wallet-Nachrichtenkanal, scannte es mit `b`,
   berechnete `ECDH(b, A)` und erkannte nur bei passendem
   Commitment seine Zahlung.
5. Bob **musste** den erkannten Output auf ein neues
   Commitment mit einem nur ihm bekannten Schlüssel
   übertragen. Erst nach Bestätigung dieser Übertragung war
   der semi-interaktive Transfer abgeschlossen.

Die Bezeichnung `PendingEnvelope` im Code benannte nur die
externe Wallet-Nachricht. Sie darf nicht als durch den
Epic-Konsens erzwingbarer Status verstanden werden.

## Was der Code tatsächlich prüfte (historisch)

- ECDH lieferte auf Alice- und Bob-Seite denselben
  abgeleiteten Blind.
- Ein anderer Scan-Schlüssel erkannte das Envelope nicht.
- Bobs verpflichtende Abschlussübertragung wechselte im
  lokalen Ledger von `Pending` zu `Claimed`.
- Ein vorheriger Alice-Übergang setzte `SenderSpent`; eine
  spätere Bob-Ausgabe wurde danach abgewiesen. Dieser Test
  dokumentierte die bewusst akzeptierte First-Spend-Wins-
  Semantik.
- Die AgSig-Autorisierung band Zahlungs-ID, Ursprung- und
  Ziel-Commitment, Betrag und Pfad aneinander.

## Ausdrückliche Nicht-Eigenschaften (historisch)

Dies war weder ein Sicherheitsbeweis noch ein
Epic-Cash-Protokoll. Insbesondere:

- `PendingEnvelope` war eine externe Wallet-Nachricht. Das
  aktuelle Epic-Output und die bestehende
  Range-Proof-Nachricht trugen den benötigten 33-Byte-
  Ephemeralschlüssel nicht nachweislich.
- Der `OneSidedLedger` baute keine Transaktion, keinen
  Kernel und keinen Range Proof. Er erzwang weder eine
  Mimblewimble-Bilanzgleichung noch UTXO-, Mempool- oder
  Reorg-Regeln.
- Der Zielschlüssel wurde nur im Modell als Bob-eigenes
  Output-Commitment dargestellt.
- Alice konnte den gemeinsamen Blind ebenfalls ableiten und
  daher vor Bobs verpflichtender Abschlussübertragung
  konkurrierend ausgeben.

## Relation zur neuen SNIP-Spezifikation

Die hier dokumentierte Modellierung **enthielt keine**
Usernet-Chain-Phase, keinen echten Konsenspfad, keinen
Multi-Zahlungs-Flow und keine Bob-only-End-Outputs. Sie
diente ausschließlich der frühen Erforschung der
Kryptografie-Grundlagen. Die heute gültige Zielsetzung
steht in [PROTOCOL.md](../PROTOCOL.md).
