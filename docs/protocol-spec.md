# Protokollspezifikation (POC)

Status: lokales Forschungsmodell, keine Epic-Cash-Konsensspezifikation.

`PaymentDescriptor` enthält Bobs Claim-Schlüssel, Alices Refund-Schlüssel und eine domänensepariert gehashte `descriptor_id`. `Ledger::create_payment` erzeugt daraus einen `PendingPayment` mit Testbetrag, Refund-Höhe, eindeutiger `payment_id` und `commitment_placeholder`. Der Platzhalter ist ein deterministischer SHA-256-Hash über kanonische Testdaten und ausdrücklich kein Pedersen Commitment.

Der Zustandsautomat ist `Pending → Claimed` oder `Pending → Refunded`; Endzustände besitzen keine ausgehenden Übergänge. Nur `Ledger` ändert einen Status.

Eine Ed25519-Autorisierung signiert domänensepariert `payment_id`, Pfad, Commitment-Platzhalter, Betrag, Refund-Höhe und Transaktionskontext. Das Ledger prüft beim Claim Bobs Schlüssel jederzeit. Beim Refund prüft es zuerst `current_height >= refund_height` und dann Alices Schlüssel. Die Prüfung und Statusänderung passieren in einem Ledger-Aufruf, sodass nur der zuerst angenommene Pfad gilt.

Dieses Modell implementiert weder Epic-Konsens noch Mimblewimble-Transaktionen, Pedersen Commitments, Range Proofs, Kernel-Signaturen, Cut-through oder Wallet-Interaktivität. Es trifft keine Aussage über reale Protokollsicherheit oder Konsenskompatibilität.
