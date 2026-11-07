# Kryptografische Grundlage (Prototyp)

Dieses Modul verwendet die lokalen Git-Submodule `upstream/epic` und
`upstream/epic-wallet` als Referenz. Beide pinnen in ihren `Cargo.lock`-Dateien
dieselbe `EpicCash/rust-secp256k1-zkp`-Revision `1131a20920576413cd949e012a160561080324c1`;
diese Revision ist auch hier explizit gepinnt. Verwendet werden ausschließlich
die getesteten Schnittstellen `Secp256k1::commit`, `commit_sum`,
`verify_commit_sum`, `ecdh::SharedSecret` und
`aggsig::{sign_single, verify_single}`.

`src/crypto.rs` kapselt die Pedersen- und AgSig-Primitiven. `src/one_sided.rs`
verwendet zusätzlich `ECDH` über `SharedSecret` zur Ableitung eines
gemeinsamen Blind. Der Blind wird mit Domain-Separator, Ephemeralschlüssel,
Receiver-Schlüssel, Betrag und Zählerwert domänengetrennt gehasht, bis ein
gültiger secp256k1-Skalar erzeugt wird.

Die AgSig-Autorisierung bindet `payment_id`, Pfad (`claim`/`sender-spend`),
Ursprungs-Commitment, Ziel-Commitment und Betrag. So kann eine
Claim-Autorisierung weder für `sender_spend` noch mit verändertem Kontext
verifiziert werden.

Es existieren absichtlich keine Blöcke, keine UTXO-Datenbank, keine
Netzwerkkommunikation, keine Wallet-Schlüsselverwaltung und keine echten
Consensus-Objekte.
