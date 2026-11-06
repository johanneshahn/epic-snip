# Isoliertes Epic-Usernet für Receipt-Claim-Experimente

Status: Infrastrukturentwurf. Dieser Ablauf nutzt ausschließlich das lokale
`upstream/epic`-Subprojekt und verbindet sich nicht mit Mainnet oder Floonet.
Er enthält einen lokalen, experimentellen Receipt-Claim-Konsenspfad. Dieser
ist nur für Usernet vorgesehen, uncommittet und weder auditiert noch
netzwerk-/mainnet-kompatibel.

## Vorhandene Unterstützung

Der vorhandene Node unterstützt `--usernet`, das intern
`ChainTypes::UserTesting` auswählt. Dafür sind lokale Standardendpunkte
vorgesehen:

## Terminal-Demo ohne Wallet

Der lokale Core enthält eine direkte Terminal-Demo für den Stealth-Receipt-
Ableitungspfad. Sie erzeugt keinen Block, baut aber eine gültige lokale
Core-Transaktion: Alice erstellt einen Receipt-Output mit gültigem Range Proof;
Bob scannt den Hint, leitet Shared-Blind und Claim-Key ab, rewound den Betrag
und baut einen ein-input/ein-output BobClaim. Die vollständige
`Transaction::validate(...)`-Prüfung erfolgt im Usernet-Modus:

```sh
cargo run -p epic_core --example receipt_stealth_demo
```

Dies ist kein Ersatz für Wallet-Scan, UTXO-/TxHashset-Verbrauch oder
Usernet-Block-End-to-End-Tests.

Der Core-Builder `libtx::build::receipt_output(...)` prüft weiterhin nur das
Receipt-Output-Format mit einem normalen Keychain-Blind. Die Terminal-Demo
nutzt zusätzlich die klar als Forschungs-API markierten Builder für den
Shared-Blind-Output und die BobClaim-Transaktion. Sie sind nicht wallet- oder
netzwerkreif.

| Dienst | Standard |
| --- | --- |
| Node-API | `127.0.0.1:23413` |
| P2P | `127.0.0.1:23414` |
| Wallet-Listener | `127.0.0.1:23415` |
| Stratum | `127.0.0.1:23416` |

`run_test_miner` ist ausdrücklich für `usertesting` vorgesehen. Für echte
Coinbase-Ausgaben benötigt er einen laufenden lokalen Wallet-Listener; ohne
diesen dürfen keine wertvollen Daten erwartet werden.

## Einrichten

Aus dem Projektwurzelverzeichnis:

```sh
mkdir -p .usernet/node
cd .usernet/node
cargo run --manifest-path ../../upstream/epic/Cargo.toml -- --usernet server config
```

In der erzeugten `epic-server.toml` ausschließlich für dieses lokale Usernet
setzen:

```toml
run_test_miner = true
test_miner_wallet_url = "http://127.0.0.1:23415"
```

Dann Node und lokalen Wallet-Listener in getrennten Terminals starten. Der
exakte Wallet-Befehl hängt vom eingecheckten `epic-wallet`-CLI-Stand ab und ist
vor Ausführung mit `cargo run -- --help` zu prüfen. Anschließend:

```sh
cd .usernet/node
cargo run --manifest-path ../../upstream/epic/Cargo.toml -- \
  --usernet server --config_file epic-server.toml run
```

Der Node darf nur an `127.0.0.1` gebunden und nie mit Mainnet-Konfiguration
gestartet werden. Die Daten unter `.usernet/` sind Entwicklungsdaten und keine
Wallet-Sicherung.

## Geplante Validierung

Nach der Core-Erweiterung muss das Usernet mindestens folgende echte Blöcke
akzeptieren bzw. ablehnen:

1. Spezial-Output mit Alice- und Bob-Receipt-Key-Hash erzeugen.
2. Gültigen `BobClaim` minen und dessen Kernel später abrufen.
3. Gültigen `AliceReclaim` in einem unabhängigen Fall minen.
4. Gewöhnlichen Kernel für einen Spezial-Input im Mempool und beim Blockprozess
   ablehnen.
5. Falschen Receipt-Key, falsche Signatur, Replay und falsches Commitment
   ablehnen.

## Automatisierter lokaler Chain-Roundtrip

Der Test `chain/tests/receipt_claim_roundtrip.rs` deckt bereits einen echten
lokalen Chain-Ablauf ohne Netzwerk ab: Er mint eine reife Usernet-Testcoinbase,
mined Alice' Receipt-Funding-Transaktion, bestätigt den Receipt-UTXO, mined
Bobs Claim in einem Folgeblock und prüft UTXO-Verbrauch, Zieloutput sowie die
vollständige TxHashset-Validierung.

```sh
cargo test -p epic_chain --test receipt_claim_roundtrip
```

Der Test startet keinen Node, keine Wallet und keinen Mempool. Daher ersetzt
er keinen Mehrprozess-Usernet-Test und belegt weder P2P- noch Mainnet-
Kompatibilität.
6. Nachfolgeblöcke minen und prüfen, dass Kernel-Beweise erhalten bleiben,
   während Cut-through weiterhin für normale Outputs arbeitet.

Diese Testfälle ergänzen die schnelleren automatisierten Chain-Tests; sie
ersetzen keine Sicherheits- oder Privacy-Begutachtung.
