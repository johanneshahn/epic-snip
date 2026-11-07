# Protokollspezifikation (POC)

Status: lokales Forschungsmodell, keine Epic-Cash-Konsensspezifikation.

`src/one_sided.rs` modelliert einen one-sided shared-redeemable-Zahlungsablauf.
Bob erzeugt einen langfristigen `ReceiverKey` (secp256k1) und veröffentlicht
den öffentlichen Scan-Schlüssel. Alice erzeugt für jede Zahlung einen neuen
Ephemeralschlüssel `a` und leitet aus `ECDH(a, B)` einen domänenseparierten
gemeinsamen Blind ab. Aus diesem Blind entsteht ein Pedersen Commitment und
ein `PendingEnvelope` mit dem ephemeralen öffentlichen Schlüssel `A`.

Bob empfängt das Envelope über einen modellierten einseitigen
Wallet-Nachrichtenkanal, scannt es mit `b`, berechnet `ECDH(b, A)` und erkennt
nur bei passendem Commitment seine Zahlung. Er erhält dann ein
`ClaimCapability` mit dem gemeinsamen Blind.

## Zustandsautomat

Der `OneSidedLedger` hat drei Zustände:

- `Pending`: Output ist noch nicht ausgegeben.
- `Claimed`: Bob hat den Output auf einen eigenen Schlüssel übertragen.
- `SenderSpent`: Alice hat den Output konkurrierend ausgegeben.

Endzustände besitzen keine ausgehenden Übergänge. Der Ledger akzeptiert nur
den ersten gültigen Pfad: `Claim XOR SenderSpend`.

## AgSig-Autorisierung

Die AgSig-Signatur bindet `payment_id`, Pfad (`claim`/`sender-spend`),
Ursprungs-Commitment, Ziel-Commitment und Betrag. Eine Claim-Autorisierung
kann nicht für `sender_spend` umgewidmet werden und umgekehrt.
