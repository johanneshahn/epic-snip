# Testplan

Die Integrationstests prüfen:

- Bob erkennt und claimt ein `OneSidedPayment` über seinen Scan-Schlüssel.
- Ein anderer Receiver-Schlüssel erkennt das Envelope nicht.
- Alice kann den Output konkurrierend ausgeben, bevor Bob claimt; ein späterer
  Bob-Claim wird dann abgewiesen (`AlreadySpent`).
- Die AgSig-Autorisierung ist an `payment_id` und Commitment gebunden.
- Pedersen Commitments haben die erwartete additive Eigenschaft.

Erwartet wird bei jeder ungültigen Autorisierung ein Fehler ohne Zustandswechsel
und bei konkurrierenden gültigen Versuchen genau ein erfolgreicher Übergang.
Nicht abgedeckt sind reale Netzwerke, Konsens, Reorgs, Side Channels,
Zufallsgüte, Persistenz, Last, DoS, formale Verifikation und echte
Mimblewimble-Kryptografie. Erfolgreiche Tests sind kein Sicherheitsnachweis.
