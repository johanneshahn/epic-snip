# Kryptografische Grundlage — überholt durch PROTOCOL.md

**Status: HISTORISCH und überholt durch
[PROTOCOL.md](../PROTOCOL.md).**

Dieses Dokument beschrieb die
Kryptografie-Grundlagen des In-Memory-POC. Es
ist **nicht** die Kryptografie-Spezifikation des
in PROTOCOL.md definierten Ziel-Protokolls.

---

## Historischer Inhalt

Dieses Modul verwendete die lokalen
Git-Submodule `upstream/epic` und
`upstream/epic-wallet` als Referenz. Beide
pinten in ihren `Cargo.lock`-Dateien dieselbe
`EpicCash/rust-secp256k1-zkp`-Revision
`1131a20920576413cd949e012a160561080324c1`;
diese Revision ist auch hier explizit
gepinnt. Verwendet wurden ausschließlich die
getesteten Schnittstellen
`Secp256k1::commit`, `commit_sum`,
`verify_commit_sum`, `ecdh::SharedSecret` und
`aggsig::{sign_single, verify_single}`.

`src/crypto.rs` kapselte die Pedersen- und
AgSig-Primitiven. `src/one_sided.rs`
verwendete zusätzlich `ECDH` über
`SharedSecret` zur Ableitung eines
gemeinsamen Blind. Der Blind wurde mit
Domain-Separator, Ephemeralschlüssel,
Receiver-Schlüssel, Betrag und Zählerwert
domänengetrennt gehasht, bis ein gültiger
secp256k1-Skalar erzeugt wurde.

Die AgSig-Autorisierung band `payment_id`,
Pfad (`claim`/`sender-spend`), Ursprungs-
Commitment, Ziel-Commitment und Betrag. So
konnte eine Claim-Autorisierung weder für
`sender_spend` noch mit verändertem Kontext
verifiziert werden.

Es existierten absichtlich keine Blöcke, keine
UTXO-Datenbank, keine Netzwerkkommunikation,
keine Wallet-Schlüsselverwaltung und keine
echten Consensus-Objekte.

## Relation zur neuen SNIP-Spezifikation

Die hier dokumentierten Primitiven sind weiterhin
die Grundlage der in `upstream/epic`
implementierten Kryptografie:

| POC-Primitiv | SNIP-Äquivalent in `upstream/epic` |
| --- | --- |
| `ReceiverKey` (secp256k1) | `receipt_spend_secret_key` + `scan_public_key` in Bobs Descriptor |
| `ECDH(a, B)` → gemeinsamer Blind | `ECDH(r_i, S)` → `h_i` + `q_i` (Receipt-Claim-Design) |
| Pedersen Commitment | Output-Commitment + Range Proof |
| AgSig (single) | `ReceiptClaim`-Signatur über `(path, original_commitment, kernel_excess, fee)` |
| `PendingEnvelope` (Wallet-Nachricht) | **Nicht mehr verwendet** — Bob scannt die Chain direkt |

Die konkrete Stealth-Ableitung und die
`BobClaim`-Konsensregeln stehen in
[receipt-claim-design.md](receipt-claim-design.md)
und in `upstream/epic/core/src/receipt_claim.rs`.
