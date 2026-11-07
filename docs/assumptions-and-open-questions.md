# Annahmen und offene Fragen — überholt durch PROTOCOL.md

**Status: HISTORISCH und überholt durch
[PROTOCOL.md](../PROTOCOL.md).**

Dieses Dokument beschrieb die Annahmen und offenen
Fragen des In-Memory-POC. Es ist **nicht** die
aktuelle Liste für die SNIP-Implementierung.

---

## Historische Annahmen

- secp256k1, Pedersen Commitments, AgSig und SHA-256
  wurden nur als etablierte Bibliotheksprimitive
  verwendet; die Protokollkonstruktion selbst war
  nicht bewiesen.
- Der Betrag war im Modell sichtbar; das Pedersen
  Commitment bot im isolierten POC keine
  Betragsvertraulichkeit.
- Eine `payment_id` band Commitment,
  Ephemeralschlüssel und Receiver-Schlüssel,
  ersetzte aber keine Konsensserialisierung oder
  Output-Bindung.
- Die ECDH-Blind-Ableitung war eine heuristische
  Konstruktion; eine formale Sicherheitsanalyse
  stand aus.
- Es fehlten Schlüsselableitung,
  Descriptor-Wiederverwendungsschutz, Backup,
  Recovery, Reorg- und Mempool-Regeln.
- Die Zuverlässigkeit des externen
  `PendingEnvelope`-Kanals wurde nicht modelliert:
  verlorene oder manipulierte Envelopes wurden
  nicht erkannt.

Vor einer produktiven Nutzung wären formale
Spezifikation und Beweise, ein konsensfähiges
Design, Integration echter Confidential
Transactions sowie unabhängige Kryptografie- und
Sicherheits-Audits erforderlich. Dieses Projekt
behauptet keine Epic-Cash-Konsenskompatibilität und
keine reale Protokollsicherheit.

## Offene Fragen für das neue SNIP-Protokoll

Die in PROTOCOL.md definierte Ziel-Implementierung
bringt folgende offene Fragen mit:

1. **Payment-Descriptor-Format.** Welche Daten
   muss der Descriptor enthalten, damit Alice
   beliebig viele Zahlungen davon ableiten kann,
   ohne Bob einzubeziehen?
   (Vorschlag: Bobs langfristiger Scan-Public-Key
   + Receipt-Spend-Public-Key + Versions- und
   Domain-Separator.)
2. **Stealth-Steuerbarkeit.** Alice wählt pro
   Zahlung einen frischen Ephemeralschlüssel. Wie
   wird die korrespondierende Stealth-Ableitung
   kanonisch definiert, damit Bob sie
   deterministisch und ohne Kommunikation
   nachvollziehen kann?
3. **Scan-Effizienz.** Wie scannt Bobs Wallet
   die Chain effizient, ohne jeden
   `Output` gegen jedes mögliche
   `descriptor_id` zu prüfen? (Vorschlag:
   Index über `H(descriptor_id)` in
   `SharedReceiptKeyHashes`.)
4. **Multi-Zahlungs-Korrelierbarkeit.** Wie wird
   verhindert, dass drei Zahlungen aus dem
   selben Descriptor öffentlich
   verknüpfbar sind (PROTOCOL.md §3.7)?
5. **Betrag-Leakage beim Scan.** Findet der
   Scan zusätzlich zum Output selbst
   Betragsinformation, die über den
   Range-Proof hinausgeht?
6. **Descriptor-Rotation.** Wie wird der
   Descriptor bei kompromittiertem
   Bobs-Schlüssel rotiert, ohne
   laufende Zahlungen zu verwerfen?
7. **Reorg-Behandlung.** Was passiert mit
   Bobs Scan-Ergebnissen, wenn ein Block
   reorged wird? (Vorschlag: Scan
   ausschließlich auf bestätigten Blöcken.)
8. **Claim-Order.** Kann Bob mehrere Claims in
   einer Transaktion bündeln, oder muss jeder
   Claim eine eigene Transaktion sein? (PROTOCOL.md
   §2.5 legt „eigene Transaktion" nahe.)
9. **End-Output-Exklusivität.** Wie wird die
   Exklusivität der End-Outputs für Bob
   kryptografisch nachgewiesen (PROTOCOL.md §3.4)?
10. **Wallet-Integration.** Wie werden
    Descriptor-Erzeugung, Scan und Claim in
    die bestehende Epic-Wallet integriert,
    ohne die Mainnet-Kompatibilität zu
    gefährden?

Diese Fragen werden im nächsten Design-Pass
behandelt. Bis dahin gilt PROTOCOL.md als
oberste Priorität.
