# Annahmen und offene Fragen

- secp256k1, Pedersen Commitments, AgSig und SHA-256 werden nur als etablierte
  Bibliotheksprimitive verwendet; die Protokollkonstruktion selbst ist nicht
  bewiesen.
- Der Betrag ist im Modell sichtbar; das Pedersen Commitment bietet im
  isolierten POC keine Betragsvertraulichkeit.
- Eine `payment_id` bindet Commitment, Ephemeralschlüssel und Receiver-Schlüssel,
  ersetzt aber keine Konsensserialisierung oder Output-Bindung.
- Die ECDH-Blind-Ableitung ist eine heuristische Konstruktion; eine formale
  Sicherheitsanalyse steht aus.
- Es fehlen Schlüsselableitung, Descriptor-Wiederverwendungsschutz, Backup,
  Recovery, Reorg- und Mempool-Regeln.
- Die Zuverlässigkeit des externen `PendingEnvelope`-Kanals wird nicht
  modelliert: verlorene oder manipulierte Envelopes werden nicht erkannt.

Vor einer produktiven Nutzung wären formale Spezifikation und Beweise, ein
konsensfähiges Design, Integration echter Confidential Transactions sowie
unabhängige Kryptografie- und Sicherheits-Audits erforderlich. Dieses Projekt
behauptet keine Epic-Cash-Konsenskompatibilität und keine reale
Protokollsicherheit.
