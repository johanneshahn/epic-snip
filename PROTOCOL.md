# Protokoll: Einseitige Wiederverwendbare Zahlungen (SNIP)

Status: Funktionale Spezifikation. Keine Konsensspezifikation.
Nicht für Mainnet, Floonet oder echte Gelder.

## Oberste Priorität — Akzeptanzkriterium

Jede weitere Dokumentation, jede Designentscheidung und jede Implementierung
dient diesem einzigen Akzeptanzkriterium. Abweichungen von diesem Kriterium
sind nur zulässig, wenn sie nachweislich zu mehr Compliance mit §3 führen.

**Bob veröffentlicht einen Payment Descriptor genau einmal.**
Dann, ohne je mit Bob zu kommunizieren oder Bobs Beteiligung
einzuziehen, erstellt **Alice** **drei unabhängige Zahlungen**
und bringt sie **alle drei** in der lokalen Usernet-Chain
zur Bestätigung. **Bob detektiert** alle drei Zahlungen
**ausschließlich durch Scannen der Chain**. **Bob claimt** jede
der drei Zahlungen in **eigener Transaktion**. **Alle drei
Claims** werden **in der Usernet-Chain bestätigt**.
**Die End-Outputs** sind **ausschließlich von Bob auszahlbar**.

Jede einzelne Anforderung aus §2 und §3 der funktionalen
Spezifikation gilt zusätzlich als bindend, soweit sie nicht
mit diesem Akzeptanzkriterium kollidiert.

## §2 Transaktionsfluss (5 Schritte)

1. **Bob: Payment Descriptor einmalig.**
   Bob erzeugt sein langfristiges Schlüsselmaterial und
   veröffentlicht den öffentlichen **Payment Descriptor**.
   Der Descriptor ist wiederverwendbar: aus ihm kann Alice
   beliebig viele Zahlungen ableiten, ohne Bob erneut
   einzubeziehen.

2. **Alice: unabhängige Zahlungen erstellen.**
   Für jede Zahlung erzeugt Alice eine eigene Transaktion
   aus dem Descriptor. Bob ist nicht online, nicht an der
   Erstellung beteiligt, erhält keine Nachricht.
   Jede Zahlung ist eine separate, unabhängige Transaktion.

3. **Alice: Bestätigung in der Usernet-Chain.**
   Alice bringt jede der drei Zahlungen in einen Block der
   lokalen Usernet-Chain und erhält Bestätigung.
   Die Bestätigung erfolgt ohne Bob.

4. **Bob: Detektion durch Scannen.**
   Bob scannt die Usernet-Chain. **Ohne irgendeine
   Kommunikation mit Alice** erkennt er jede der drei
   Zahlungen als an sich adressiert. Er kann aus der Chain
   ableiten:
   - *dass* eine Zahlung an ihn gerichtet ist,
   - *welchen Betrag* die Zahlung hat.
   Bob ist während Alices Erstellung offline.
   Bob muss nicht teilnehmen, *aber* muss er die Zahlungen
   **finden können**.

5. **Bob: Claim in eigener Transaktion.**
   Bob claimt jede der drei Zahlungen in einer eigenen
   Transaktion auf einen Output, der **ausschließlich von
   Bob auszahlbar** ist.
   Der Claim enthält **keine** Alice-Beteiligung.
   Alle drei Claims werden in der Usernet-Chain bestätigt.
   Die End-Outputs sind ausschließlich Bob's.

## §3 Bindende Anforderungen

| Nr | Anforderung |
| --- | --- |
| 3.1 | Bob muss in der Lage sein, eingehende Zahlungen zu **erkennen**, ohne mit Alice zu kommunizieren. |
| 3.2 | Bob muss in der Lage sein, **Beträge** der an ihn gerichteten Zahlungen zu bestimmen. |
| 3.3 | Bob muss Zahlungen **ohne Alice** claimen können. |
| 3.4 | Die End-Outputs müssen **ausschließlich von Bob** auszahlbar sein. |
| 3.5 | Alle Transaktionen müssen den **Epic-Cash-Konsens** der lokalen Usernet-Chain erfüllen. |
| 3.6 | **Vertraulichkeit und Privatsphäre** müssen gemäß dem EPIC-Projekt erhalten bleiben. |
| 3.7 | Es darf **keine überflüssig öffentlich verknüpfbare Information** auf der Chain hinterlassen werden. |
| 3.8 | **Alice kann prüfen, dass Bob die Shared Outputs übernommen hat.** Alice muss in der Lage sein, aus der Chain nachzuvollziehen, dass Bob eine Zahlung claimt hat und die Mittel nun in seinen End-Outputs stecken. |
| 3.9 | **Alice kann die Shared Outputs jederzeit selbst beanspruchen, solange Bob sie nicht in seine eigenen Outputs übertragen hat.** Der Alice-Reclaim-Pfad ist ein konkurrierender Pfad zum Bob-Claim. Der erste bestätigte gültige Spend gewinnt (First-Confirmed-Spend-Wins). |

## §4 Nicht verbindlich — freie Designentscheidungen

Die folgende Punkte sind **bewusst nicht** von der
Funktional-Spezifikation vorgeschrieben. Jede
Designentscheidung zu diesen Punkten ist zulässig, solange
sie §2, §3 und §5 erfüllt:

- Gemeinsame Outputs / Split-Outputs
- Die konkrete Schlüsselableitung
- Zusätzliche Kernel-Features
- Konsenserweiterungen jenseits dessen, was §3.5 erfordert

**Nicht mehr frei:** Der `BobClaim`-Kernel ist **Teil
des Zielprotokolls** (§3.3). Der Alice-Reclaim-Pfad
nutzt die **bestehende Standard-Mechanik von Epic Cash**
(gewöhnlicher `Plain`-Kernel) und muss nicht neu
erfunden werden (§3.9).

## §5 Akzeptanzkriterium (wörtlich)

> "Die vollständige Demonstration des Transaktionsflusses:
> Bob erstellt den Descriptor einmalig → Alice erstellt 3
> unabhängige Zahlungen (Bob nicht beteiligt) → alle 3
> werden in der lokalen Usernet-Chain bestätigt → Bob
> detektiert alle 3 → Bob claimt jede durch seine eigene
> Transaktion → alle 3 Claims werden bestätigt → End-Outputs
> ausschließlich von Bob auszahlbar."
>
> Zusätzlich:
> - Alice kann aus der Chain verifizieren, dass Bob die
>   Zahlungen übernommen hat (§3.8).
> - Alice kann die Shared Outputs jederzeit selbst
>   beanspruchen, solange Bob sie nicht bereits
>   übertragen hat (§3.9).

## §6 Grundsatz

> "Ein erfolgreicher isolierter Krypto- oder Konsens-Test
> **ersetzt nicht** die vollständige Demonstration des
> Transaktionsflusses."

Jede einzelne Komponente — Krypto, Konsens, Wallet-Scan,
Claim — muss **innerhalb der vollständigen
3-Zahlungs-Demonstration** nachweislich funktionieren.

## Statusbegriff

Alle Dokumente in diesem Repo, die sich auf die *vorherige*
Forschung (one-sided shared-redeemable POC,
`AliceReclaim`, `BobClaim`-Attributionsforschung) beziehen,
sind **abgeschlossen und überholt durch diese Spezifikation**.
Sie bleiben als historische Aufzeichnung erhalten; ihr Inhalt
ist **keine** Anleitung für weitere Entwicklung.
