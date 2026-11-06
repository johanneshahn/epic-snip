# Bedrohungsmodell

## Angenommene Garantien des Modells

- Der Besitz von Bobs privatem Schlüssel ist für einen Claim nötig.
- Der Besitz von Alices privatem Schlüssel **und** das Erreichen von `refund_height` sind für einen Refund nötig.
- Vertragseigenschaften können nicht nach der Signatur verändert werden.
- Eine Autorisierung ist an einen Spend und den gewählten Pfad gebunden.
- Der Validator verarbeitet den Verbrauch eines Outputs atomar.

## Nicht abgedeckt

- Konsens- und Implementierungsfehler in Epic Cash oder Abweichungen durch Forks/Reorgs.
- Diebstahl oder Verlust von Schlüsseln, kompromittierte Endgeräte und Wallet-Backups.
- DoS, Zensur, Gebührenmarkt und Mempool-Policy.
- Betragsvertraulichkeit: `commitment` ist nur ein Platzhalter.
- Seitenkanäle, Zufallszahlengüte, Serialisierungsfehler und kryptografische Audits.
- Schutz gegen einen bösartigen Sender vor der Aufnahme eines Outputs in eine reale Blockchain.

## Sicherheitskritische Regel

Die zusätzliche Signatur darf in einer echten Chain nicht bloße Metadaten sein. Sie muss Teil der Konsensvalidierung des Input-Pfads sein und dauerhaft an den konkreten Output gebunden werden. Andernfalls könnte Alice vorzeitig ausgeben oder ein Angreifer eine Autorisierung wiederverwenden.
