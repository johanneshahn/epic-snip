# Protokoll: One-Sided Shared-Redeemable Payment

Status: Forschungsentwurf, Version 2. Keine Konsensspezifikation.

## Ziel

Alice erzeugt ohne Bobs Online-Beteiligung einen Pending Output, der für
Bob automatisch erkennbar und von beiden Parteien gemeinsam ausgabbar ist.
Bob **muss** den erkannten Output auf einen nur ihm bekannten Schlüssel
übertragen, um den semi-interaktiven Transfer abzuschließen. Alice kann
konkurrierend ausgeben, bis Bob diese Übertragung vorgenommen hat — ein
bewusst akzeptiertes Risiko des nicht-interaktiven Ablaufs.

## Vertragsdaten

`PendingEnvelope` enthält:

| Feld | Zweck |
| --- | --- |
| `payment_id` | SHA-256 über Commitment, Ephemeralschlüssel und Receiver-Schlüssel |
| `amount` | Zahlungsbetrag (simuliert, keine CT) |
| `commitment` | Pedersen Commitment `amount * H + blind * G` |
| `ephemeral_public_key` | Alices ephemeraler öffentlicher Schlüssel `A` |
| `receiver_public_key` | Bobs langfristiger Scan-Schlüssel `B` |
| `shared_spend_public_key` | Öffentliches Gegenstück zum gemeinsamen Blind |

`payment_id = SHA256(DOMAIN || "id\|" || commitment || A_compressed || B_compressed)`.

Der gemeinsame Blind wird aus `ECDH(a, B)` bzw. `ECDH(b, A)` abgeleitet und
mit A, B, Betrag und Domain-Separator domänengetrennt gehasht. Nur Bob kann
ihn korrekt ableiten.

## Validierung

1. Bob empfängt das `PendingEnvelope` über einen modellierten Wallet-Nachrichtenkanal.
2. Bob berechnet `ECDH(b, A)`, leitet den gemeinsamen Blind ab und prüft, ob das abgeleitete Commitment mit dem im Envelope übereinstimmt.
3. Bei Übereinstimmung erhält Bob ein `ClaimCapability` mit dem gemeinsamen Blind.
4. Bob registriert das Payment im `OneSidedLedger` und kann es jederzeit per AgSig-Autorisierung claimen.
5. Alice kann alternativ `sender_spend` aufrufen, um den Output selbst auszugeben — dies gewinnt den Race, bis Bob geclaimt hat.

Damit gilt im Modell: `Claim XOR SenderSpend`, da ein Output nur einmal ausgegeben werden kann.

## Noch für Epic zu entwerfen

Eine reale Umsetzung muss die Vertragsbindung in Output, Range Proof und
Kernel integrieren und sie in jedem Node validieren. Offene Punkte sind
insbesondere:

- sichere Abbildung des `PendingEnvelope` auf Epics Output-/Range-Proof-Format;
- Erhalt der Regeln unter Cut-through und Aggregation;
- Konsensserialisierung, Aktivierung und Netzwerkmigration;
- Mempool-Konflikte, Reorgs und atomare UTXO-Updates;
- Wallet-Scan-Schlüsselverwaltung, Erkennung und Wiederherstellung;
- unabhängige Kryptographie- und Konsens-Audits.

Ohne diese Arbeit gibt es keinerlei Aussage zur Epic-Cash-Konsenskompatibilität.
