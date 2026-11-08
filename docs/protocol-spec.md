# Protokollspezifikation — überholt durch PROTOCOL.md

**Status: HISTORISCH und überholt durch
[PROTOCOL.md](../PROTOCOL.md).**

Dieses Dokument beschrieb die frühe In-Memory-POC-Modellierung
in `src/one_sided.rs`. Es ist **nicht** die heute gültige
Protokollbeschreibung. Die oberste Priorität und die
vollständige Funktionalspezifikation stehen in
[PROTOCOL.md](../PROTOCOL.md).

---

## Historischer Inhalt

### Modell (In-Memory, keine Chain)

`src/one_sided.rs` modellierte einen one-sided
shared-redeemable-Zahlungsablauf. Bob erzeugte einen
langfristigen `ReceiverKey` (secp256k1) und veröffentlichte
den öffentlichen Scan-Schlüssel. Alice erzeugte für jede
Zahlung einen neuen Ephemeralschlüssel `a` und leitete aus
`ECDH(a, B)` einen domänenseparierten gemeinsamen Blind ab.
Aus diesem Blind entstand ein Pedersen Commitment und ein
`PendingEnvelope` mit dem ephemeralen öffentlichen Schlüssel
`A`.

Bob empfing das Envelope über einen modellierten einseitigen
Wallet-Nachrichtenkanal, scannte es mit `b`, berechnete
`ECDH(b, A)` und erkannte nur bei passendem Commitment seine
Zahlung. Er erhielt dann ein `ClaimCapability` mit dem
gemeinsamen Blind.

### Zustandsautomat (In-Memory)

Der `OneSidedLedger` hatte drei Zustände:

- `Pending`: Output ist noch nicht ausgegeben.
- `Claimed`: Bob hat den Output auf einen eigenen Schlüssel
  übertragen.
- `SenderSpent`: Alice hat den Output konkurrierend
  ausgegeben.

Endzustände besitzen keine ausgehenden Übergänge. Der Ledger
akzeptierte nur den ersten gültigen Pfad: `Claim XOR
SenderSpend`.

### AgSig-Autorisierung

Die AgSig-Signatur band `payment_id`, Pfad
(`claim`/`sender-spend`), Ursprungs-Commitment,
Ziel-Commitment und Betrag. Eine Claim-Autorisierung konnte
nicht für `sender_spend` umgewidmet werden und umgekehrt.

## Warum überholt

Diese In-Memory-Spezifikation erfüllte **keine** der in
PROTOCOL.md §3 definierten bindenden Anforderungen:

| Anforderung (PROTOCOL.md §3) | POC-Erfüllung |
| --- | --- |
| 3.5 Epic-Cash-Konsens | ❌ kein Konsens |
| 3.2 Bob bestimmt Betrag aus der Chain | ⚠️ nur via Wallet-Nachricht |
| 3.1 Bob erkennt ohne Kommunikation | ⚠️ via Wallet-Nachricht, nicht Chain-Scan |
| 3.7 Keine überflüssig verknüpfbare Info | ⚠️ Wallet-Nachricht enthält Zahlungsdetail |

Die heute gültige Spezifikation verlangt die
vollständige Demonstration in der Usernet-Chain
ohne externe Wallet-Nachrichten.
