# Isoliertes Epic-Usernet für Receipt-Claim-Experimente

Status: Infrastrukturentwurf. Dieser Ablauf nutzt ausschließlich das lokale
`upstream/epic`-Subprojekt und verbindet sich nicht mit Mainnet oder Floonet.
Er implementiert noch keinen Receipt-Claim-Konsenspfad.

## Vorhandene Unterstützung

Der vorhandene Node unterstützt `--usernet`, das intern
`ChainTypes::UserTesting` auswählt. Dafür sind lokale Standardendpunkte
vorgesehen:

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
6. Nachfolgeblöcke minen und prüfen, dass Kernel-Beweise erhalten bleiben,
   während Cut-through weiterhin für normale Outputs arbeitet.

Diese Testfälle ergänzen die schnelleren automatisierten Chain-Tests; sie
ersetzen keine Sicherheits- oder Privacy-Begutachtung.
