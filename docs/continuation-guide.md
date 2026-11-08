# Übergabe: Weiterarbeit an Epic SNIP — überholt

**Status: HISTORISCH und überholt durch
[PROTOCOL.md](../PROTOCOL.md).**

Dieses Dokument beschrieb den Übergabepunkt von der
In-Memory-POC-Phase zur Chain-Phase. Es ist **nicht**
mehr gültig. Die heute aktive Arbeit wird in
[CURRENT_STEP.md](../CURRENT_STEP.md) geführt.

---

## Historischer Inhalt

### Ausgangspunkt (historisch)

Der POC modellierte einen one-sided
shared-redeemable-Zahlungsablauf. Alice erzeugte ohne
Antwort von Bob einen Output, den Bob über einen
veröffentlichten Scan-Schlüssel und eine externe
Wallet-Nachricht erkennen konnte. Alice und Bob
leiteten beide eine gemeinsame Spend-Fähigkeit ab. Bob
musste den Output auf einen ausschließlich von ihm
kontrollierten Schlüssel übertragen. Erst dessen
Bestätigung schloss den semi-interaktiven Transfer ab.

Der Code unter `src/one_sided.rs` und die Tests unter
`tests/` prüften diese Eigenschaften nur im
Arbeitsspeicher. Sie modellierten weder einen UTXO-Satz
noch eine echte Mimblewimble-Transaktion.

Es gab bewusst keinen Konsensstatus „pending" und keine
Anforderung, Alice vor dieser verpflichtenden
Abschlussübertragung auszusperren.

### Weiterarbeit (historisch)

Die Weiterarbeit in `upstream/epic` umfasste:

- Usernet-Chain-Roundtrip (1 Zahlung)
- BobClaim-Konsenspfad
- Spend-Attribution-Forschung
- AliceReclaim-Entfernung
- Verdict **VERIFIED**

Alle diese Punkte sind **abgeschlossen** und in ihren
jeweiligen Dokumenten dokumentiert.

## Übergang zur neuen SNIP-Spezifikation

Die heutige Aufgabe ist die vollständige Demonstration
des 3-Zahlungs-Flows gemäß
[PROTOCOL.md](../PROTOCOL.md) §5. Der genaue Status,
die offenen Fragen und die nächsten Schritte stehen in
[CURRENT_STEP.md](../CURRENT_STEP.md).
