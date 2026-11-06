# Receipt-Claim: atomarer Empfangsnachweis ohne Offenlegung des Zieloutputs

Status: Entwurfsnotiz, nicht implementiert, nicht konsenskompatibel und nicht
kryptografisch bewiesen.

## Ziel

Der von Alice erkennbar gemachte gemeinsame Output darf nur über einen
nachweisbaren Auflösungspfad ausgegeben werden. Bob muss ihn auf einen eigenen
Schlüssel übertragen, damit der semi-interaktive Transfer abgeschlossen ist.
Bob darf diese Übertragung nicht durchführen können, ohne gleichzeitig einen
für Alice nachweisbaren Claim zu hinterlassen. Entsprechend darf Alice ihn
nicht gewöhnlich ausgeben können, ohne einen für Bob nachweisbaren Reclaim zu
hinterlassen.

Zugleich soll die Chain nicht offenlegen, welcher spätere Zieloutput Bob gehört.
Ein Außenstehender soll daher nur erkennen können, dass ein spezieller Output
geclaimt wurde; nicht, welcher neue Output der endgültige Bob-Output ist.

## Rollen und private Vorbedingung

Für jede Zahlung erzeugen Alice und Bob jeweils einen frischen Receipt-
Schlüssel `A_i` bzw. `B_i`. Vor der Zahlung gibt Bob Alice eine private
Delegation:

```text
Delegation_i = Sign_BobProofIdentity(
  domain || payment_id || B_i || Ablauf-/Versionsdaten
)
```

Alice hält eine entsprechende, selbst signierte lokale Zuordnung für `A_i`.
Die Bob-Delegation wird nicht on-chain veröffentlicht. Sie erlaubt Alice später
im Streitfall, `B_i` Bobs Proof-Identität zuzuordnen. Pro Zahlung neue Keys
vermeiden öffentliche Verknüpfbarkeit mehrerer Claims über eine statische
Payment-Proof-Adresse.

## Vorgeschlagenes Konsensformat

Ein neuer Spezial-Output bindet bei seiner Entstehung mindestens:

```text
alice_receipt_key_hash = H(domain || A_i)
bob_receipt_key_hash   = H(domain || B_i)
```

Zwei neue Kernel-Varianten enthalten mindestens:

```text
AliceReclaim:
  original_commitment = C
  receipt_public_key = A_i
  receipt_signature = Sign_Ai(domain || C || kernel_binding || "alice-reclaim")

BobClaim:
  original_commitment = C
  receipt_public_key = B_i
  receipt_signature = Sign_Bi(domain || C || kernel_binding || "bob-claim")
```

Während der Validierung muss der Node prüfen:

1. `C` ist ein als Input dieser Transaktion ausgegebener Spezial-Output.
2. Der dort gespeicherte Key-Hash entspricht dem Key der jeweiligen Variante.
3. `receipt_signature` über den kanonisch definierten Bindungswert ist gültig.
4. Alle normalen Regeln für Transaktionsbilanz, Range Proofs, Fee und
   Kernel-Signatur erfüllen sich ebenfalls.
5. Jede Ausgabe eines Spezial-Outputs enthält genau einen passenden
   `AliceReclaim`- oder `BobClaim`-Kernel. Ein gewöhnlicher Kernel darf einen
   solchen Output nicht ausgeben.

Der genaue `claim_kernel_binding` muss vor jeder Implementierung kanonisch
spezifiziert werden. Er muss Replay in einer anderen Transaktion ausschließen,
darf aber keinen endgültigen Bob-Zieloutput im Kernel offenlegen.

## Was öffentlich und privat bleibt

| Information | Sichtbarkeit |
| --- | --- |
| Spezial-Output `C` wurde geclaimt | dauerhaft öffentlich über den Receipt-Claim-Kernel |
| Ein einmaliger Receipt-Key hat aufgelöst | dauerhaft öffentlich |
| Bobs dauerhafte Proof-Identität | privat, außer Alice legt `Delegation_i` offen |
| Bobs endgültiger Zieloutput | nicht Bestandteil des Receipt-Claim-Kernels; nur Bob erkennt ihn beim Wallet-Scan |
| Betrag | weiterhin durch Commitment und Range Proof verborgen, sofern keine Partei ihn offenlegt |

Der Claim reduziert dennoch die Graph-Privatsphäre für `C`: Seine Ausgabe ist
nach Cut-through dauerhaft mit einem Receipt-Claim-Kernel verbunden. Das ist
der gezielte Preis für den langlebigen Nachweis. Die Zuordnung zum späteren
Zieloutput und zu weiteren Bob-Claims wird dadurch nicht automatisch öffentlich.

## Atomare Eigenschaft

Ein Bob-Claim ist nur gültig, wenn seine Receipt-Signatur im gültigen
`BobClaim`-Kernel enthalten ist. Ein Alice-Reclaim ist analog nur mit einem
`AliceReclaim`-Kernel gültig. Daher kann keine Seite den Spezial-Output
ausgeben, ohne den jeweiligen Auflösungsnachweis on-chain zu hinterlassen.
Alice kombiniert den bestätigten Bob-Kernel mit ihrer privaten Delegation, um
die Zuordnung zu Bob zu belegen.

Eine bloß extern übermittelte Quittung erfüllt dieses Ziel nicht: Bob könnte die
Transaktion senden, ohne Alice die Quittung zu geben.

## Offene Fragen

- Wie wird der Spezial-Output serialisiert, versioniert und aktiviert, ohne
  normale Outputs zu beeinflussen?
- Welche Kernel-Daten bleiben nach Aggregation und Cut-through sicher erhalten?
- Wie wird verhindert, dass eine Transaktion mehrere Auflösungskernel oder
  fremde Spezial-Inputs missbräuchlich kombiniert?
- Wie werden Reorgs, Mempool-Konflikte, Replay und wiederverwendete
  Receipt-Keys behandelt?
- Welche Auswirkungen haben beobachtbare Spezial-Outputs auf die Anonymitäts-
  menge und auf Netzwerk-Metadaten?
- Welche unabhängige Kryptografie- und Konsensprüfung ist vor jeder
  produktionsnahen Aussage erforderlich?
