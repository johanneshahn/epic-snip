# Annahmen und offene Fragen

- Ed25519 und SHA-256 werden nur als etablierte Bibliotheksprimitive verwendet; die Protokollkonstruktion selbst ist nicht bewiesen.
- Blockhöhen sind monoton simulierte Ganzzahlen, keine Blockchain-Historie.
- Der Betrag ist sichtbar und `commitment_placeholder` keine kryptografische Wertbindung im Sinn von Pedersen Commitments.
- Eine `payment_id` bindet Testdaten, ersetzt aber keine Konsensserialisierung oder Output-Bindung.
- Es fehlen Schlüsselableitung, Descriptor-Wiederverwendungsschutz, Backup, Recovery, Reorg- und Mempool-Regeln.

Vor einer produktiven Nutzung wären formale Spezifikation und Beweise, ein konsensfähiges Design, Integration echter Confidential Transactions sowie unabhängige Kryptografie- und Sicherheits-Audits erforderlich. Dieses Projekt behauptet keine Epic-Cash-Konsenskompatibilität und keine reale Protokollsicherheit.
