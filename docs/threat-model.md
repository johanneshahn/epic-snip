# Bedrohungsmodell

Angreifer können `PendingEnvelope`-Nachrichten manipulieren, Payment-IDs
anpassen oder gleichzeitig von Alice und Bob ausgeben. Der Modell lehnt
manipulierte oder an einen anderen Pfad gebundene AgSig-Signaturen ab. Die
zentrale Zustandsänderung verhindert eine zweite Ausgabe im selben Ledger.

Vorausgesetzt werden die Unfälschbarkeit von secp256k1-AgSig, der Schutz der
privaten Keys (Alice's Ephemeralschlüssel, Bobs Receiver-Schlüssel) und eine
atomare Ausführung eines `OneSidedLedger`-Aufrufs. Schlüsselverlust oder
-diebstahl, DoS, Zensur, Datenrennen ohne geeignete äußere Synchronisierung,
Serialisierungsfehler, Hash-Kollisionen und Speichererschöpfung sind nicht
gelöst. Bei paralleler Benutzung muss der Aufrufer das Ledger beispielsweise
mit `Mutex` schützen.

Nicht modelliert sind Netzwerk, Mempool, Reorgs, Gebühren, UTXO-Konsens,
reale Wallets, Zuverlässigkeit des `PendingEnvelope`-Kanals und
Betragsvertraulichkeit. Dies ist keine formale Sicherheitsanalyse und keine
Aussage zu Epic Cash oder Mimblewimble.
