# Release parameter setup: trust choice

Status: the user selected a single-operator setup for the first release to prioritize delivery. A dedicated fixed setup for the final authenticated circuit is still required; disposable experiment parameters remain ineligible for release. No project-operated server is required.

## One setup operator for the first release

The user runs a dedicated setup command on their own machine. It draws fresh randomness from the operating system and saves the public proving key, verifying key, and fingerprints. The game never generates or accepts a replacement key while proving a run. The deployed reward contract pins the approved verifier.

Participants must trust that the setup operator and machine did not retain or expose the secret setup randomness. Retained setup secrets can allow forged proofs and dishonest reward claims. A successful benchmark does not remove this assumption. Deleting a file alone cannot prove that the secrets were destroyed.

This is the accepted implementation path with the existing Rust proving code. The active trust assumption belongs in technical and release documentation. It is not a multi-party ceremony or trustless setup. If a later release changes the setup process, its whitepaper should describe the process actually in use; obsolete implementation history need not be a feature of that document.

## Deferred alternative: multi-party ceremony

Use an established, independently verifiable Groth16 ceremony process after freezing the circuit. Security relies on at least one participant contributing honestly and discarding their secret contribution. No server is inherently required: participants can exchange public transcript files.

This requires a verified circuit export and key-import/proving path, transcript verification, and participant contributions. The existing experiment has not implemented that interoperability. Existing [snarkjs ceremony tools](https://github.com/iden3/snarkjs) and [arkworks Circom compatibility code](https://github.com/arkworks-rs/circom-compat) are implementation references, not installed or validated dependencies for DiggerChain.

## Common release requirements

- Bind the circuit, rules, generator, public-input order, and verifier artifacts to published fingerprints.
- Compare finalized constraint matrices for key compatibility; retain source provenance separately so presentation-only updates do not require a new setup.
- Validate proofs produced with the release parameters natively, locally on an EVM, and on Robinhood testnet.
- Keep setup randomness out of save files, logs, public artifacts, and player proving workflows.
- Obtain final approval of the concrete deployment and trust assumptions before live funding.

The user choosing a model authorizes its implementation, not publication, signing, or funding.
