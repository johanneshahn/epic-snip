# Testplan

Die Integrationstests prüfen erfolgreichen Claim, frühesten zulässigen Refund, Claim nach Ablauf, die gegenseitige Exklusivität, fremde Schlüssel, Pfad- und Zahlungs-Replay, manipulierte Bindungsdaten, doppelte Einreichung und zwei konkurrierende Aufrufe. Zusätzlich prüft ein Test monotone Höhen.

Erwartet wird bei jeder ungültigen Autorisierung ein Fehler ohne Zustandswechsel und bei konkurrierenden gültigen Versuchen genau ein erfolgreicher Übergang. Nicht abgedeckt sind reale Netzwerke, Konsens, Reorgs, Side Channels, Zufallsgüte, Persistenz, Last, DoS, formale Verifikation und echte Mimblewimble-Kryptografie. Erfolgreiche Tests sind kein Sicherheitsnachweis.
