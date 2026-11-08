# Threat Model

## Assumed Guarantees of the Model

- Only the holder of the shared blind (Bob after a successful scan) can
  claim the output.
- Alice can spend the output competitively as long as Bob has not
  yet claimed — this is a conscious trade-off of the non-interactive
  flow.
- The AgSig authorization is bound to `payment_id`, path (`claim`/
  `sender-spend`), commitment, and destination commitment.
- The `OneSidedLedger` processes the spending of an output atomically.

## Not Covered

- Consensus and implementation errors in Epic Cash or deviations due to
  forks/reorgs.
- Theft or loss of keys, compromised endpoints and wallet backups.
- DoS, censorship, fee market and mempool policy.
- Reliability of the external `PendingEnvelope` delivery channel: a lost
  or tampered envelope cannot be detected.
- Side channels, randomness quality, serialization errors, and
  cryptographic audits.
- Protection against a malicious sender before an output is included in a
  real blockchain.
- Formal security analysis of the ECDH blind derivation.

## Security-Critical Rule

The AgSig authorization must not be mere metadata in a real chain.
It must be part of the consensus validation of the input path and
permanently bound to the concrete output. Otherwise an attacker could
reuse an authorization or Alice could spend competitively without
Bob being able to detect it.