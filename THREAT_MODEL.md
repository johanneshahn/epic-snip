# Bedrohungsmodell

## Angenommene Garantien des Modells

- Nur der Besitzer des gemeinsamen Blind (Bob nach erfolgreichem Scan) kann
  den Output claimen.
- Alice kann den Output konkurrierend ausgeben, solange Bob noch nicht
  geclaimt hat — dies ist ein bewusster trade-off des nicht-interaktiven
  Ablaufs.
- Die AgSig-Autorisierung ist an `payment_id`, Pfad (`claim`/`sender-spend`),
  Commitment und Ziel-Commitment gebunden.
- Der `OneSidedLedger` verarbeitet den Verbrauch eines Outputs atomar.

## Nicht abgedeckt

- Konsens- und Implementierungsfehler in Epic Cash oder Abweichungen durch Forks/Reorgs.
- Diebstahl oder Verlust von Schlüsseln, kompromittierte Endgeräte und Wallet-Backups.
- DoS, Zensur, Gebührenmarkt und Mempool-Policy.
- Die Zuverlässigkeit des externen `PendingEnvelope`-Zustellkanals: verloren
  oder manipuliert werden kann das Envelope nicht erkannt werden.
- Seitenkanäle, Zufallszahlengüte, Serialisierungsfehler und kryptografische Audits.
- Schutz gegen einen bösartigen Sender vor der Aufnahme eines Outputs in eine
  reale Blockchain.
- Formale Sicherheitsanalyse der ECDH-Blind-Ableitung.

## Sicherheitskritische Regel

Die AgSig-Autorisierung darf in einer echten Chain nicht bloße Metadaten
sein. Sie muss Teil der Konsensvalidierung des Input-Pfads sein und dauerhaft
an den konkreten Output gebunden werden. Andernfalls könnte ein Angreifer eine
Autorisierung wiederverwenden oder Alice könnte konkurrierend ausgeben, ohne
dass Bob dies erkennen kann.
