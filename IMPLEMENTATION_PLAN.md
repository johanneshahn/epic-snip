# Implementierungsplan: Epic SNIP Forschungsprototyp

## Ziel

Erstelle im bestehenden Projektordner einen nachvollziehbaren kryptografischen Forschungsprototypen für ein hypothetisches Epic-Cash-SNIP-Protokoll:

- Alice erzeugt eine Zahlung ohne Bobs Online-Beteiligung.
- Bob kann die Zahlung jederzeit claimen.
- Alice kann erst ab einer definierten Blockhöhe einen Refund ausführen.
- Genau einer der beiden Pfade kann erfolgreich sein.

Wichtig: Dies ist ausschließlich eine lokale Modellierung und Testumgebung. Sie ist keine Epic-Cash-Wallet, keine Blockchain-Implementierung, nicht konsenskompatibel und darf keine echten Schlüssel, Gelder oder Netzwerke verwenden.

## Technische Entscheidungen

- Sprache: Rust
- Build/Test: Cargo
- Modell: deterministische In-Memory-Ledger-Simulation
- Kryptografie: zunächst Schlüssel- und Signatur-Wrapper aus einer etablerten Rust-Bibliothek; keine selbst entwickelte Kryptografie
- Zeit: ausschließlich simulierte Blockhöhen, keine Systemzeit
- Persistenz: keine; Testdaten nur im Arbeitsspeicher
- Netzwerk: keines

## Verzeichnisstruktur

```text
./
├── README.md
├── Cargo.toml
├── LICENSE
├── .gitignore
├── docs/
│   ├── protocol-spec.md
│   ├── threat-model.md
│   ├── assumptions-and-open-questions.md
│   └── test-plan.md
├── src/
│   ├── lib.rs
│   ├── types.rs
│   ├── descriptor.rs
│   ├── payment.rs
│   ├── authorization.rs
│   ├── ledger.rs
│   ├── errors.rs
│   └── test_support.rs
└── tests/
    ├── claim_flow.rs
    ├── refund_flow.rs
    ├── exclusivity.rs
    ├── authorization.rs
    └── adversarial_cases.rs
```

## Umsetzungsstand

Die Projektstruktur wird direkt in diesem Workspace geführt (kein zusätzlicher
`repo/`-Unterordner). Die Implementierung verwendet ein deterministisches
In-Memory-Ledger, Ed25519 aus `ed25519-dalek`, simulierte Blockhöhen und keine
Netzwerk- oder Persistenzfunktionen. Details und die Grenzen des Modells stehen
in `README.md` und `docs/`.

## Domänenmodell

Implementiere mindestens diese Typen:

```rust
PaymentDescriptor {
    receiver_public_key,
    refund_public_key,
    descriptor_id,
}

PendingPayment {
    payment_id,
    amount,                 // Nur Testwert, keine echte Confidential Transaction
    receiver_public_key,
    refund_public_key,
    refund_height,
    status,                 // Pending | Claimed | Refunded
    commitment_placeholder, // Deutlich als Platzhalter markieren
}

SpendAuthorization {
    payment_id,
    path,                   // Claim | Refund
    signature,
}

Ledger {
    current_height,
    payments,
}
```

`commitment_placeholder` darf ausdrücklich keine Pedersen-Commitment-Sicherheit vortäuschen. Für den ersten Prototyp genügt ein deterministischer Hash über kanonisch serialisierte Testdaten, dokumentiert als Nicht-Konsensmodell.

## Protokollregeln

1. Alice erhält einmalig Bobs `PaymentDescriptor`.
2. Alice erzeugt einen `PendingPayment` mit Betrag und `refund_height`.
3. Bob kann vor oder nach `refund_height` mit seinem privaten Schlüssel einen Claim autorisieren.
4. Alice darf einen Refund erst bei `current_height >= refund_height` autorisieren.
5. Das Ledger akzeptiert nur eine gültige, bisher nicht ausgegebene Zahlung.
6. Nach einem erfolgreichen Claim oder Refund ist der andere Pfad endgültig ungültig.
7. Signaturen müssen mindestens an `payment_id`, Ausgabepfad und relevante Transaktionsdaten gebunden sein, damit sie nicht für einen anderen Output oder Pfad wiederverwendet werden können.

## Implementierungsphasen

### Phase 1: Projektgrundlage

- Rust-Bibliothek mit Cargo anlegen.
- Linter und Formatierung konfigurieren.
- README mit Sicherheitswarnung und Ausführungsanleitung schreiben.
- Öffentliche API klein halten: `create_payment`, `claim`, `refund`, `advance_height`.

Erfolgskriterium: `cargo test` läuft und ein leerer Test besteht.

### Phase 2: Deterministisches Ledger

- `Ledger` mit Blockhöhe und Payment-Speicher implementieren.
- Payment-Erzeugung implementieren.
- Zustandsübergänge zentral im Ledger validieren.
- Keine direkte Manipulation des Payment-Status außerhalb des Ledgers erlauben.

Erfolgskriterium: Ein Pending Payment lässt sich erstellen und wieder auslesen.

### Phase 3: Autorisierung

- Test-Schlüsselpaare erzeugen.
- Claim-Autorisierung mit Bobs Schlüssel verifizieren.
- Refund-Autorisierung mit Alices Schlüssel verifizieren.
- Signaturnachricht domänenseparieren, z. B. `EPIC-SNIP-POC-v0|payment_id|path|refund_height`.

Erfolgskriterium: Falsche Schlüssel, manipulierte Payment-IDs und manipulierte Pfade werden abgewiesen.

### Phase 4: Zeitlogik und Exklusivität

- Refund vor Ablauf ablehnen.
- Refund ab der Refund-Höhe erlauben.
- Claim jederzeit erlauben, solange die Zahlung unspent ist.
- Nach erfolgreichem Claim jeden Refund ablehnen.
- Nach erfolgreichem Refund jeden Claim ablehnen.

Erfolgskriterium: Die Ledger-Transition ist atomar und lässt nie beide Ausgabepfade zu.

### Phase 5: Dokumentation

Erstelle folgende Dokumente:

- `docs/protocol-spec.md`: Nachrichtentypen, Zustandsautomat, Validierungsregeln und Abgrenzung zum Epic-Konsens.
- `docs/threat-model.md`: Angreifer, Annahmen, Replay, Manipulation, Schlüsselverlust, Race Conditions und DoS.
- `docs/assumptions-and-open-questions.md`: alle unbelegten Aussagen und fehlenden kryptografischen Beweise.
- `docs/test-plan.md`: Testfälle, erwartete Ergebnisse und bewusst nicht abgedeckte Risiken.

### Phase 6: Tests

Mindestens diese Tests implementieren:

1. Bob kann eine gültige Zahlung claimen.
2. Alice kann vor `refund_height` nicht refunden.
3. Alice kann bei `refund_height` refunden.
4. Bob kann nach `refund_height` weiterhin claimen, sofern Alice nicht zuerst refunded hat.
5. Claim nach Refund wird abgelehnt.
6. Refund nach Claim wird abgelehnt.
7. Fremder Schlüssel kann weder claimen noch refunden.
8. Claim-Signatur kann nicht als Refund-Signatur wiederverwendet werden.
9. Signatur für Payment A gilt nicht für Payment B.
10. Manipulierte Refund-Höhe oder Payment-ID wird abgelehnt.
11. Doppelte Einreichung derselben Ausgabe wird abgelehnt.
12. Zwei konkurrierende Ausgaben führen zu genau einem erfolgreichen Zustandsübergang.

## Dokumentationspflichten

Das README und jedes Architekturpapier müssen klar festhalten:

- Keine Aussage zur Konsenskompatibilität mit Epic Cash.
- Keine Aussage zur Sicherheit von Mimblewimble-Transaktionen.
- Keine vollständige Umsetzung von Pedersen Commitments, Range Proofs, Kernel-Signaturen, Cut-through oder Wallet-Interaktivität.
- Keine formale Sicherheitsanalyse und kein kryptografischer Beweis.
- Die Tests belegen nur Verhalten des simulierten Zustandsmodells, keine reale Protokollsicherheit.
- Vor einer produktiven Verwendung wären unabhängige kryptografische Begutachtung, formale Spezifikation, Konsensdesign und Audits nötig.

## Abnahmekriterien

- `cargo fmt --check` erfolgreich.
- `cargo test` erfolgreich.
- Jeder Zustandsübergang ist durch Tests abgedeckt.
- README beschreibt Start, Test und Grenzen.
- Bedrohungsmodell und offene Fragen sind vorhanden.
- Kein Code, Dokument oder Test behauptet Konsenskompatibilität oder reale kryptografische Sicherheit.
