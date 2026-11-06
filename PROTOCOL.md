# Protokoll: Pending Payment (Variante B)

Status: Forschungsentwurf, Version 1. Keine Konsensspezifikation.

## Ziel

Alice kann mit Bobs einmalig erhaltenem öffentlichen Schlüssel einen Pending Output erzeugen. Bob kann ihn jederzeit übernehmen. Alice erhält erst ab `refund_height` ein alternatives Refund-Recht. Der erste gültig bestätigte Spend verbraucht den Output.

## Vertragsdaten

`PendingPaymentOutput` enthält:

| Feld | Zweck |
| --- | --- |
| `version` | Protokollversion |
| `commitment` | Platzhalter für den vertraulichen Wert-Commitment |
| `receiver_pubkey` | Bobs Claim-Schlüssel |
| `refund_pubkey` | Alices Refund-Schlüssel |
| `refund_height` | absolute Blockhöhe, ab der Refund zulässig ist |
| `payment_id` | SHA-256 über alle obigen Vertragsbedingungen |
| `creation_signature` | Alices Signatur über `payment_id` |

`payment_id = SHA256("epic-snip/pending-payment/v1/contract" || version || commitment || receiver_pubkey || refund_pubkey || LE64(refund_height))`.

Die Implementierung nutzt kanonische Byte-Reihenfolgen und Domain Separation. Jede Ausgabesignatur bindet außerdem `payment_id`, Pfad (`claim`/`refund`) und die Hash-ID der ausgebenden Transaktion. Sie kann deshalb nicht in eine andere Transaktion oder auf den anderen Pfad kopiert werden.

## Validierung

1. Versionsnummer, `payment_id` und Alices Erzeugungssignatur prüfen.
2. Sicherstellen, dass der Output noch nicht ausgegeben ist.
3. Bei `Claim`: Bobs Signatur über die Spend-Autorisierung prüfen.
4. Bei `Refund`: `block_height >= refund_height` prüfen und dann Alices Signatur prüfen.
5. Output atomar als ausgegeben markieren.

Damit gilt im Modell: `Claim XOR Refund`, da ein UTXO nur einmal konsumiert werden kann.

## Noch für Epic zu entwerfen

Eine reale Umsetzung muss die Vertragsbindung in Output, Range Proof und Kernel integrieren und sie in jedem Node validieren. Offene Punkte sind insbesondere:

- sichere Abbildung auf Epics verwendete Kurve und Signaturen;
- Erhalt der Regeln unter Cut-through und Aggregation;
- Konsensserialisierung, Aktivierung und Netzwerkmigration;
- Mempool-Konflikte, Reorgs und atomare UTXO-Updates;
- Wallet-Descriptor, Erkennung, Schlüsselableitung und Wiederherstellung;
- unabhängige Kryptographie- und Konsens-Audits.

Ohne diese Arbeit gibt es keinerlei Aussage zur Epic-Cash-Konsenskompatibilität.
