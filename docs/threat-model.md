# Bedrohungsmodell — überholt durch PROTOCOL.md

**Status: HISTORISCH und überholt durch
[PROTOCOL.md](../PROTOCOL.md).**

Dieses Dokument beschrieb das Bedrohungsmodell des
In-Memory-POC. Es ist **nicht** das Bedrohungsmodell
des in PROTOCOL.md definierten Ziel-Protokolls.

---

## Historischer Inhalt

Angreifer konnten `PendingEnvelope`-Nachrichten
manipulieren, Payment-IDs anpassen oder gleichzeitig
von Alice und Bob ausgeben. Der In-Memory-Ledger lehnte
manipulierte oder an einen anderen Pfad gebundene
AgSig-Signaturen ab. Die zentrale Zustandsänderung
verhinderte eine zweite Ausgabe im selben Ledger.

Vorausgesetzt wurden die Unfälschbarkeit von
secp256k1-AgSig, der Schutz der privaten Keys
(Alice's Ephemeralschlüssel, Bobs Receiver-Schlüssel)
und eine atomare Ausführung eines
`OneSidedLedger`-Aufrufs. Schlüsselverlust oder
-diebstahl, DoS, Zensur, Datenrennen ohne geeignete
äußere Synchronisierung, Serialisierungsfehler,
Hash-Kollisionen und Speichererschöpfung waren nicht
gelöst. Bei paralleler Benutzung musste der Aufrufer
das Ledger beispielsweise mit `Mutex` schützen.

Nicht modelliert waren Netzwerk, Mempool, Reorgs,
Gebühren, UTXO-Konsens, reale Wallets,
Zuverlässigkeit des `PendingEnvelope`-Kanals und
Betragsvertraulichkeit. Dies war keine formale
Sicherheitsanalyse und keine Aussage zu Epic Cash
oder Mimblewimble.

## Offene Bedrohungsfragen für das neue SNIP-Protokoll

Die in PROTOCOL.md definierte Ziel-Implementierung
bringt neue Bedrohungsfragen mit, die noch nicht
analyse- und dokumentationspflichtig sind:

- **Descriptor-Leakage:** Kann ein Angreifer aus dem
  Payment Descriptor auf Bobs langfristiges
  Schlüsselmaterial schließen?
- **Multi-Zahlungs-Korrelation:** Sind die drei
  unabhängigen Zahlungen öffentlich
  verknüpfbar (PROTOCOL.md §3.7)?
- **Scan-Korrelierbarkeit:** Kann ein
  Beobachter Bobs Scan-Aktivität mit
  Alices Zahlungen korrelieren?
- **Claim-Korrelierbarkeit:** Lässt sich Bobs
  Claim-Transaktion zu einem bestimmten
  Zahlungsausgang zuordnen?
- **Descriptor-Wiederverwendung:** Wie viele
  Zahlungen kann Alice aus einem Descriptor
  ableiten, bevor eine Wiederverwendungs-
  Signatur auffällt?
- **Betrag-Leakage:** Entdeckt der Scan
  zusätzlich zum Output selbst
  Betragsinformation, die über den
  Range-Proof hinausgeht?
- **Descriptor-Rotation:** Wie wird der
  Descriptor bei kompromittiertem
  Bobs-Schlüssel rotiert, ohne
  laufende Zahlungen zu verwerfen?

Diese Fragen werden in der
Forschungsphase nach der vollständigen
3-Zahlungs-Demonstration behandelt. Sie sind
**nicht** Teil der aktuellen Akzeptanz.
