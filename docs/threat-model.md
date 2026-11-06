# Bedrohungsmodell

Angreifer können Nachrichten, Payment-IDs, Pfade, Höhen und Transaktionskontexte manipulieren, alte Autorisierungen wieder einreichen oder gleichzeitig Claim und Refund versuchen. Das Modell weist manipulierte, an einen anderen Pfad oder eine andere Zahlung gebundene Signaturen ab. Die zentrale Zustandsänderung verhindert eine zweite Ausgabe im selben Ledger.

Vorausgesetzt werden die Unfälschbarkeit von Ed25519, der Schutz der privaten Testschlüssel und eine atomare Ausführung eines `Ledger`-Aufrufs. Schlüsselverlust oder -diebstahl, DoS, Zensur, Datenrennen ohne geeignete äußere Synchronisierung, Serialisierungsfehler, Hash-Kollisionen und Speichererschöpfung sind nicht gelöst. Bei paralleler Benutzung muss der Aufrufer das Ledger beispielsweise mit `Mutex` schützen.

Nicht modelliert sind Netzwerk, Mempool, Reorgs, Gebühren, UTXO-Konsens, reale Wallets und Betragsvertraulichkeit. Dies ist keine formale Sicherheitsanalyse und keine Aussage zu Epic Cash oder Mimblewimble.
