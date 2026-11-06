# Kryptografische Grundlage (Prototyp)

Dieses Modul verwendet die lokalen Git-Submodule `upstream/epic` und `upstream/epic-wallet` als Referenz. Beide pinnen in ihren `Cargo.lock`-Dateien dieselbe `EpicCash/rust-secp256k1-zkp`-Revision `1131a20920576413cd949e012a160561080324c1`; diese Revision ist auch hier explizit gepinnt. Verwendet werden ausschließlich die getesteten Schnittstellen `Secp256k1::commit`, `commit_sum`, `verify_commit_sum` und `aggsig::{sign_single, verify_single}`. Die Testmuster lehnen sich an deren Commitment-Summen- und AgSig-Tests an.

`src/crypto.rs` kapselt diese Primitiven. Ein Pedersen Commitment folgt `value * H + blind * G`; die Signatur ist eine single-signer AgSig/Schnorr-Autorisierung. Vor dem Signieren wird ein SHA-256-Transcript mit Domain Separation gebildet. Er bindet Payment-ID, Pfad, das echte Commitment, Refund-Höhe und den lokalen Transaktionskontext. So kann eine Claim-Signatur weder als Refund noch mit verändertem Kontext verifiziert werden.

Es existieren absichtlich keine Blöcke, keine UTXO-Datenbank, keine Netzwerkkommunikation, keine Wallet-Schlüsselverwaltung und keine echten Transaktionen. Auch Range Proofs, Kernel-Excess, Kernel-Signaturen, Cut-through und Konsensregeln sind **nicht** Teil dieser Stufe. Das Commitment allein beweist keinen Wertebereich; ohne Range Proof kann es nicht als vollständige Mimblewimble-Ausgabe betrachtet werden.

Die Abhängigkeit ist auf die Upstream-Revision gepinnt, aber noch nicht unabhängig auditiert. Diese Arbeit behauptet weder Epic-Cash-Konsenskompatibilität noch Produktionssicherheit.
