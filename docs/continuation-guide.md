# Übergabe: Weiterarbeit an Epic SNIP

Status: Forschungsnotiz. Dieses Dokument beschreibt den überprüften lokalen
Stand und eine sichere Reihenfolge für die weitere Untersuchung. Es ist keine
Konsensspezifikation, kein Sicherheitsbeweis und keine Anleitung zum Einsatz
mit echten Geldern.

## Ausgangspunkt

Der POC modelliert einen lokalen Zustandsautomaten mit zwei Ausgabepfaden:

- Bob kann einen `PendingPayment` mit seinem Claim-Schlüssel ausgeben.
- Alice kann denselben Payment erst ab `refund_height` mit ihrem Refund-Schlüssel
  ausgeben.
- Der lokale `Ledger` akzeptiert nur den ersten gültigen Pfad.

Der Code unter `src/` und die Tests unter `tests/` prüfen diese Eigenschaften
nur im Arbeitsspeicher. Sie modellieren weder einen UTXO-Satz noch eine echte
Mimblewimble-Transaktion.

## Aktuelle Designentscheidung: one-sided shared-redeemable

Neben dem älteren Claim/Refund-Modell enthält `src/one_sided.rs` einen
separaten Forschungs-POC. Alice erzeugt ohne Antwort von Bob einen Output, den
Bob über einen veröffentlichten Scan-Schlüssel und eine externe
Wallet-Nachricht erkennen kann. Alice und Bob leiten beide eine gemeinsame
Spend-Fähigkeit ab. Bob muss den Output auf einen ausschließlich von ihm
kontrollierten Schlüssel übertragen. Erst dessen Bestätigung schließt den
semi-interaktiven Transfer ab.

Es gibt bewusst keinen Konsensstatus „pending“ und keine Anforderung, Alice
vor dieser verpflichtenden Abschlussübertragung auszusperren. Alice darf vorher
konkurrierend ausgeben; Bob hat bis zur Bestätigung seiner Übertragung keine
Verfügbarkeitsgarantie.
Der erste gültige Spend gewinnt. Diese Eigenschaften gelten im POC als
akzeptiertes Überweisungsrisiko, vergleichbar mit einer nicht rückholbaren
Fehlüberweisung. Sie sind nicht als Fehler des Modells zu behandeln.

Die verbleibende Forschungsfrage ist nicht die Autorisierung beider Parteien,
sondern die Empfängererkennung in einer echten Epic-Transaktion: Das POC nutzt
ein externes `PendingEnvelope`; es ist kein Epic-Output und kein
Konsensdatenfeld. Ein Payment Proof wäre erst nach einer finalen echten
Transaktion als Beleg nützlich und ersetzt dieses Erkennungsformat nicht.

## Neuer Entwurf: Receipt-Claim ohne Zieloutput-Verknüpfung

Ein künftiger Konsenspfad soll jede Auflösung des Spezial-Outputs beweisen,
ohne den endgültigen Wallet-Output öffentlich zu benennen. Dazu bindet der
Spezial-Output je einen einmaligen Alice- und Bob-Receipt-Key-Hash. Ein
`AliceReclaim`- bzw. `BobClaim`-Kernel bindet den ursprünglichen Output und den
passenden Receipt-Key mit Signatur. Gewöhnliche Kernel dürfen den Spezial-Output
nicht ausgeben. Die neuen Zieloutputs bleiben reguläre vertrauliche Outputs.

Alice erhält vor der Zahlung eine von Bob signierte, private Zuordnung des
einmaligen Receipt-Keys zu Bobs Proof-Identität. Der vorgesehene Wallet-Pfad
ist eine Stealth-Receipt-Ableitung: Bob gibt einmalig einen Scan-/Spend-
Descriptor, Alice erzeugt pro Zahlung einen frischen Hint und kann damit einen
neuen Bob-Receipt-Public-Key bilden; nur Bob leitet den privaten Claim-Key ab.
Sie kann die Delegation nur im Streitfall offenlegen. Außenstehende sehen
dadurch weder Bobs wiederverwendete Identität noch den Zieloutput. Details,
Annahmen und noch offene
Validierungsfragen stehen in `docs/receipt-claim-design.md`.

## Geprüfte Upstream-Referenzen

Die folgenden Submodule dienen als lesbare Referenz. Änderungen an ihnen sind
nicht Teil dieses POC.

| Projekt | Lokaler Pfad | geprüfter Commit |
| --- | --- | --- |
| Epic Node | `upstream/epic` | Basis `bd3a8c8c5e957e57574364326dfb17051d0096ae`, lokale experimentelle Änderungen vorhanden |
| Epic Wallet | `upstream/epic-wallet` | `845b3fcf4df12ed0139929c62157015447a9d063` |

Vor jeder Weiterarbeit Commit-Stand und lokale Änderungen mit `git submodule
status` und `git -C upstream/<projekt> status --short` kontrollieren. Die
lokalen Änderungen in `upstream/epic` sind absichtlich uncommittete
Forschungsarbeit und dürfen nicht an die EpicCash-Upstreams gepusht werden.

## Aktueller experimenteller Core-Stand

Der lokale Node enthält inzwischen einen begrenzten Usernet-Experimentpfad:

- Receipt-Outputs binden gehashte, einmalige Alice-/Bob-Receipt-Public-Keys
  und einen 33-Byte-Stealth-Hint in die Output-Identität ein.
- Der Core testet eine experimentelle ECDH-/Tweak-Ableitung: Alice erzeugt
  daraus einen frischen Bob-Receipt-Public-Key; Bob erhält aus demselben Hint
  den passenden privaten Key. Eine Epic-Wallet verwendet diese Primitive noch
  nicht beim echten Chain-Scan.
- Alice und Bob leiten zusätzlich denselben, domain-separierten Spend-Blind
  für den Receipt-Output ab. Der lokale Core-Roundtrip baut damit einen
  gültigen Receipt-Range-Proof, Bob rewound dessen Betrag und erzeugt eine
  vollständig validierbare BobClaim-Transaktion inklusive Kernel-Summe.
- Zwei Kernel-Features (`AliceReclaim`, `BobClaim`) transportieren eine
  domain-separierte Receipt-Signatur, welche Original-Commitment,
  Kernel-Excess, Fee und Pfad bindet.
- Die Transaktionsprüfung erzwingt aktuell die Eins-zu-eins-Zuordnung eines
  Receipt-Inputs zu einem passenden Resolution-Kernel sowie den registrierten
  Key-Hash. Normale Kernel können einen Receipt-Input daher nicht mehr allein
  auflösen.
- Der Pfad wird außerhalb von `UserTesting` abgelehnt. Die bestehenden
  normalen Features bleiben unverändert.
- Der TxHashset-UTXO-View und der Spend-Pfad vergleichen Receipt-Inputs mit
  der vollständigen MMR-Output-Identität, nicht nur mit dem Commitment. Der
  Mempool validiert die Receipt-Regeln vor seiner bestehenden Chain-UTXO-Prüfung.

Gezielt geprüft wurden Core-Receipt-Tests, einschließlich eines vollständigen
Receipt-Output-zu-BobClaim-Transaktionsroundtrips, `epic_chain`-Library-Tests
und der bestehende Pool-Integrationstest. Das ist lediglich eine lokale
Test-Aussage und keine End-to-End-Konsensvalidierung.

Noch fehlt: Durchlauf durch Chain/TxHashset, Mempool, Blockerzeugung,
Netzwerkversionierung, Wallet-API und ein echter Usernet-Ablauf. Bis diese
Schichten implementiert und getestet sind, dürfen die neuen Formate weder als
netzwerkfähig noch als sicher bezeichnet werden.

## Zentraler Befund

Im geprüften Node-Code besitzt `OutputFeatures` nur `Plain` und `Coinbase`
(`upstream/epic/core/src/core/transaction.rs`). Epic unterstützt außerdem
`KernelFeatures::HeightLocked`, aber dieser Lock gehört zu einem Kernel und
damit zu einer Transaktion. Er enthält weder zwei unterschiedliche
Berechtigungsschlüssel noch erzwingt er einen Claim-von-Bob- oder
Refund-von-Alice-Pfad für einen konkreten Output.

Folglich lässt sich das gewünschte SNIP-Verhalten nicht als bloße
Wallet-Erweiterung oder durch Wiederverwendung eines bestehenden Kernel-Locks
behaupten. Eine reale Variante benötigt eine ausdrücklich spezifizierte und
konsensvalidierte Erweiterung — oder einen nachvollziehbaren Beweis, dass eine
bestehende Konstruktion alle Anforderungen erfüllt. Ein solcher Beweis liegt
hier nicht vor.

## Relevante Einstiegspunkte

| Frage | Ausgangspunkt |
| --- | --- |
| Output- und Kernel-Formate | `upstream/epic/core/src/core/transaction.rs` |
| Range Proofs und Commitments | `upstream/epic/core/src/libtx/proof.rs` |
| Aggregierte Signaturen | `upstream/epic/core/src/libtx/aggsig.rs` |
| UTXO-Index und Ausgabeverbrauch | `upstream/epic/chain/src/txhashset/txhashset.rs` |
| Wallet-Transaktionsablauf | `upstream/epic-wallet/libwallet/src/slate.rs` |
| Wallet-Empfang und Finalisierung | `upstream/epic-wallet/controller/src/command.rs` |

Der Wallet-Code verwendet Slates als interaktiven Ablauf. Daraus folgt nicht,
dass ein einmaliger Descriptor für beliebig viele nicht-interaktive Zahlungen
ausreichend ist. Dieser Punkt bleibt zu spezifizieren und zu testen.

## Offene technische Risiken

1. **Konsensbindung:** Vertragsbedingungen und Autorisierung müssen dauerhaft
   an den konkreten Output gebunden werden. Zusätzliche Metadaten außerhalb der
   Konsensvalidierung reichen nicht aus.
2. **Exklusivität:** Der Node muss bei Claim und Refund denselben UTXO-Verbrauch
   atomar prüfen; Mempool-Konflikte und Reorgs dürfen keinen zweiten gültigen
   Pfad erzeugen.
3. **Cut-through und Aggregation:** Es ist zu zeigen, dass benötigte Daten und
   Nachweise unter diesen Mimblewimble-Eigenschaften erhalten und validierbar
   bleiben.
4. **Vertraulichkeit:** Ein Commitment ohne Range Proof genügt nicht. Auch ein
   gültiger Range Proof löst die Autorisierung zweier Pfade nicht automatisch.
5. **Schlüsselverwaltung:** Descriptor-Wiederverwendung, Ableitung, Backup,
   Wiederherstellung, Schlüsselverlust und Empfängererkennung sind offen.
6. **Signaturkonstruktion:** Der POC testet isolierte Primitiven, liefert aber
   keinen Sicherheitsbeweis für ein zusammengesetztes Protokoll.

## Reproduzierbarkeit vor weiterer Forschung

`Cargo.toml` verwendet aktuell eine revisionsgepinnte Git-Abhängigkeit auf
`EpicCash/rust-secp256k1-zkp` (`1131a209…`). Vor weiterer Forschung sollten
Herkunft, Revision, Lizenz und eine reproduzierbare Offline-Strategie geprüft
und dokumentiert werden.

## Empfohlene Reihenfolge

1. Die fachliche Spezifikation schreiben: Teilnehmer, Nachrichten,
   Vertragsdaten, Canonical Encoding, Spend-Transkript, Fehlerfälle und
   Sicherheitsziele.
2. Ein Angreifermodell für Mempool, Reorgs, Front-running, Replay,
   Schlüsselkompromittierung und Descriptor-Wiederverwendung erweitern.
3. Ein Datenmodell für einen möglichen neuen Output-/Input-Pfad erstellen,
   ohne ihn als kompatibel zu bezeichnen.
4. Für dieses Modell Serialisierungs-, Hash- und negative Testvektoren
   definieren, bevor Node-Code verändert wird.
5. Erst danach die Auswirkungen auf `core`, `chain`, Pool und P2P isoliert
   untersuchen; die Wallet-Änderungen folgen dem finalen Konsensentwurf.
6. Vor jeder produktionsnahen Aussage externe Kryptografie- und
   Konsensbegutachtung einplanen.

## Definition of Done für die nächste Forschungsstufe

Die nächste Stufe ist erst abgeschlossen, wenn eine versionierte Spezifikation
mindestens Datenformate, Autorisierungsnachweise, Validierungsregeln,
Aktivierung, Reorg- und Mempool-Semantik sowie negative Testvektoren enthält.
Sie muss explizit darlegen, welche Eigenschaften bewiesen, angenommen oder
noch offen sind. Bis dahin ist jede Implementierung ausschließlich ein
Forschungsartefakt.
