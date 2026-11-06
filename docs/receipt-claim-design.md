# Receipt-Claim: atomarer Empfangsnachweis ohne Offenlegung des Zieloutputs

Status: experimentelle, lokale Usernet-Implementierung in Arbeit. Weder
konsenskompatibel noch kryptografisch bewiesen; nicht für Mainnet, Floonet oder
echte Gelder geeignet.

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

## Rollen und einmaliges Setup

Bob veröffentlicht einmalig einen versionierten Stealth-Receipt-Descriptor mit
einem öffentlichen Scan-Key und einem öffentlichen Receipt-Spend-Key. Seine
zugehörigen privaten Schlüssel verbleiben ausschließlich in Bobs Wallet. Alice
speichert den Descriptor; sie benötigt pro Zahlung keine neue Antwort von Bob.

Für jede Zahlung erzeugt Alice einen frischen Reclaim-Schlüssel `A_i` sowie
einen frischen Ephemeralschlüssel `r_i`. Aus Bobs Descriptor und `r_i` erzeugt
sie einen einmaligen Bob-Receipt-Public-Key `B_i`, ohne dessen privaten
Schlüssel zu kennen. Vor oder beim Setup gibt Bob Alice eine private
Delegation:

```text
Delegation_i = Sign_BobProofIdentity(
  domain || descriptor_id || BobReceiptSpendKey || Ablauf-/Versionsdaten
)
```

Alice hält eine entsprechende lokale Zuordnung für `A_i`, `r_i` und die
Zahlungskennung. Die Bob-Delegation wird nicht on-chain veröffentlicht. Sie
erlaubt Alice später im Streitfall, den aus diesem Descriptor abgeleiteten
`B_i` Bobs Proof-Identität zuzuordnen. Frische Schlüssel pro Zahlung vermeiden
eine öffentliche Verknüpfbarkeit mehrerer Claims über eine statische
Payment-Proof-Adresse.

## Stealth-Receipt-Ableitung

Die lokale Forschungsimplementierung verwendet zwei voneinander
domain-separierte ECDH-basierte Ableitungen. Sie ist kein kryptografischer
Beweis und keine endgültige Protokollspezifikation; die folgende Notation
beschreibt den implementierten Forschungsansatz:

```text
Bob-Descriptor: ScanPublic S = sG, ReceiptSpendPublic P = pG
Alice pro Zahlung: r_i zufällig, Hint R_i = r_iG
gemeinsamer Wert: h_i = H(domain || ECDH(r_i, S) || R_i || descriptor_id)
Bob-Receipt-Public-Key: B_i = P + h_iG
Bob-Receipt-Private-Key: b_i = p + h_i

Shared-Spend-Blind: q_i = HashToScalar(
  separate_domain || ECDH(r_i, S) || R_i || S
)
```

Bob scannt `R_i`, berechnet `ECDH(s, R_i)` und damit `b_i`; Alice kann zwar
`B_i`, nicht aber `b_i` berechnen. Die gleiche ECDH-Ausgabe lässt beide Seiten
`q_i` ableiten: Alice kennt `r_i`, Bob kennt `s`. Daraus können beide dieselbe
Commitment-Blindierung für den gemeinsamen Spezial-Output bilden. Die
Ableitung für `q_i` ist explizit von der für `h_i` getrennt, damit ein
Receipt-Autorisierungsschlüssel nicht als Spend-Blind wiederverwendet wird.

`R_i` liegt in der lokalen Implementierung als kanonisch serialisierter
33-Byte-Stealth-Hint am Receipt-Output vor. Der Core baut im lokalen
Forschungs-API einen Receipt-Output mit `q_i`, rewound dessen Betrag für Bob
und erstellt daraus eine vollständig durch `Transaction::validate(...)`
prüfbare BobClaim-Transaktion. Dieser Ablauf hat keinen Standard-Wallet-
Proof-Message-, Backup- oder Recovery-Pfad und ersetzt keine Wallet-
Integration.

Als konservativer Übergangs-Fallback ist ein vorab getauschter Pool einmaliger
Bob-Receipt-Public-Keys möglich. Ein gemeinsamer Authenticator-Seed oder ein
geteiltes privates HD-Root ist dagegen unzulässig: Könnte Alice daraus `b_i`
ableiten, könnte sie einen `BobClaim` fälschen.

## Vorgeschlagenes Konsensformat

Ein neuer Spezial-Output bindet bei seiner Entstehung mindestens:

```text
alice_receipt_key_hash = H(domain || A_i)
bob_receipt_key_hash   = H(domain || B_i)
stealth_hint            = R_i oder verschlüsselter, gleichwertiger Scan-Hint
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

## Lokaler Implementierungsstand

Im lokalen Subprojekt `upstream/epic` ist ein erster Usernet-only Core-Schritt
implementiert und durch Core-Unit-Tests abgedeckt:

- `OutputFeatures::Receipt` trägt zwei Hashes einmaliger Receipt-Public-Keys
  sowie einen kanonisch geprüften 33-Byte-Stealth-Hint; Input, Output und
  MMR-Output-Identifier serialisieren diese Daten mit.
- Der Core enthält ein experimentelles ECDH-/Tweak-Primitiv: Alice kann aus
  Bobs öffentlichem Descriptor und einem frischen Ephemeralschlüssel einen
  Bob-Receipt-Public-Key bilden, während Bob aus Hint und seinen privaten
  Descriptor-Schlüsseln den zugehörigen privaten Receipt-Key ableitet.
- Der Core leitet außerdem einen getrennten gemeinsamen Spend-Blind ab. Alice
  und Bob erhalten daraus unabhängig dasselbe Commitment für den
  Spezial-Output. Der Forschungs-Builder erstellt daraus einen gültigen
  Receipt-Range-Proof; Bob kann dessen Betrag mit einem aus dem Shared-Blind
  abgeleiteten Rewind-Nonce lesen.
- Ein lokaler Roundtrip baut anschließend eine ein-input/ein-output
  BobClaim-Transaktion mit normalem Zieloutput, gültigem Range Proof,
  Kernel-Signatur, Receipt-Signatur und korrekter Kernel-Summe. Der Test ruft
  `Transaction::validate(...)` im Usernet-Modus auf.
- `libtx::build::receipt_output(...)` erzeugt einen experimentellen
  Receipt-Output mit Key-Hashes und Hint; der reguläre Proof-Builder erzeugt
  weiterhin dessen Range Proof.
- `KernelFeatures::AliceReclaim` und `KernelFeatures::BobClaim` tragen die
  jeweilige Fee und ein `ReceiptClaim`-Transkript.
- Das Transkript bindet per domain-separierter Signatur Pfad, ursprüngliches
  Commitment, Kernel-Excess und Fee. Der Node prüft die normale Kernel-Signatur
  und anschließend diese Bindung.
- Eine Transaktions-Body-Prüfung verlangt eine Eins-zu-eins-Zuordnung zwischen
  jedem ausgegebenen Receipt-Input und einem passenden Resolution-Kernel samt
  registriertem Receipt-Key-Hash.
- Receipt-Features werden in dieser experimentellen Stufe außerhalb des
  Usernet-Modus abgelehnt.

Dies ist noch keine vollständige Chain-Integration: P2P-Versionierung,
Blockerzeugung und Wallet-Erzeugung bzw. -Scan sind noch nicht als
End-to-End-Feature getestet. Insbesondere folgt aus den Unit-Tests keine
Aussage zur Konsenssicherheit oder zur sicheren Aktivierung in einem
bestehenden Netzwerk.

`libtx::build::receipt_output(...)` ist weiterhin nur ein Format-Builder mit
normalem Keychain-Blind. Daneben existieren ausdrücklich als Forschungs-API
`build_shared_receipt_output(...)` und
`build_bob_claim_transaction(...)`. Diese koppeln den Shared-Blind an
Range-Proof und Kernel-Build-Pfad und erlauben den lokalen Roundtrip. Sie
haben noch keine Wallet-Schlüsselverwaltung, keine dauerhafte Speicherung,
keinen Recovery-Entwurf und keine Usernet-Blockintegration.

Der erste TxHashset-/Pool-Schritt ist lokal vorhanden: Der UTXO-View und der
Spend-Pfad rekonstruieren beim MMR-Vergleich den vollständigen
`OutputIdentifier` aus dem Input. Dadurch sind die Receipt-Key-Hashes Teil der
UTXO-Identität; ein Input mit richtigem Commitment, aber anderen Receipt-Hashes
wird abgelehnt. Der bestehende Mempool führt vor der UTXO-Prüfung bereits
`tx.validate(...)` aus und übernimmt damit die Receipt-Claim-Validierung.
Vollständige Block- und Usernet-End-to-End-Tests stehen weiterhin aus.

## Was öffentlich und privat bleibt

| Information | Sichtbarkeit |
| --- | --- |
| Spezial-Output `C` wurde geclaimt | dauerhaft öffentlich über den Receipt-Claim-Kernel |
| Ein einmaliger Receipt-Key hat aufgelöst | dauerhaft öffentlich |
| Bobs dauerhafte Proof-Identität | privat, außer Alice legt `Delegation_i` offen |
| Bobs endgültiger Zieloutput | nicht Bestandteil des Receipt-Claim-Kernels; nur Bob erkennt ihn beim Wallet-Scan |
| Stealth-Hint `R_i` | öffentlich, aber pro Zahlung frisch; ohne Bobs Scan-Key keine öffentliche Zuordnung zu `B_i` |
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
- Wie werden Stealth-Descriptor, Hint, ECDH-Ausgabe, Hash-to-Scalar,
  Schlüsselrotation und Recovery kanonisch definiert und unabhängig geprüft?
- Welche Auswirkungen haben beobachtbare Spezial-Outputs auf die Anonymitäts-
  menge und auf Netzwerk-Metadaten?
- Welche unabhängige Kryptografie- und Konsensprüfung ist vor jeder
  produktionsnahen Aussage erforderlich?
