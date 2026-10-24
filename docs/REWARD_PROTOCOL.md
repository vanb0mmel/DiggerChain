# Bounded v1 reward protocol

Status: candidate with 62 passing user-run checks, fixed setup, a real saved-game ownership proof, and matching testnet deployments. The [first reward smoke record](../contracts/measurements/2026-10-21-smoke/README.md) contains no successful registration or payout. On 2026-10-22 the maintainer deferred the remaining reward-flow and recovery checks to accelerate launch. They remain unverified, not release prerequisites. Proof checks and contract rules are unchanged.

## Offline expedition

The canonical rules are `vault::RULES_VERSION = 2`, generator `seeded-castle-v1`, three 19 by 13 floors, and at most 512 successful actions. Charge starts at 8, capacity is 12, soft/hard digging costs 2/3, and a flask adds 8 up to capacity. Crystal values are 1/4/16 by depth, with maximum expedition score 63. Walking and portals are free; items and terrain persist through revisits. Invalid actions are atomic. [Player instructions](EXPEDITIONS.md) cover saves and controls.

An expedition starts with a random 32-byte private secret. Its public identity is `SHA256(secret)`. The first eight identity bytes interpreted big-endian select the 64-bit world seed. A save contains format version 1, rules version 2, that secret, the seed, ordered action codes, and a sealed flag. Loading checks identity and replays all actions. Explicit-seed practice saves have no reward identity. Different secrets are not a guarantee of globally unique layouts; grinding, inspection, and automated solving remain allowed.

An expedition may be sealed at any valid prefix, including after exhausting charge or reaching the action bound. It earns no submission when its score is zero. Local sealing prevents accidental further play, while the contract's identity nullifier enforces one accepted result across all its epochs. Altering a local sealed flag cannot bypass that on-chain limit.

## Proof statement and parameters

The separate `diggerchain-proof` binary checks the private SHA-256 preimage, derives its seed, and enforces the same generated gameplay constraints as M2. Public inputs, in fixed order, are rules version, seed, chain ID, epoch, identity high 128 bits, score, charge, floor, x, y, successful actions, identity low 128 bits, recipient, and settlement contract. Chain and epoch fit 64 bits; recipient and contract fit 160 bits. Both identity halves use big-endian byte order.

The proof establishes knowledge of the expedition secret and a legal route to the result under the chosen public context. It does not establish human effort, real elapsed time, the exact original play history, or that play happened during the claimed epoch. A proof copied from another participant cannot select a different recipient. The contract also requires the transaction sender to equal that recipient.

The release operator runs one dedicated fixed Groth16/BN254 setup after the circuit is frozen. Its public proving key, verifying key, verifier source, and manifest fingerprints are distributed with the matching proof tool. Proving loads and validates those artifacts; it never runs setup or silently substitutes a new key. The deployed settlement contract pins the verifier. The accepted [single-operator trust assumption](SETUP_MODEL.md) remains active until an actually deployed replacement changes it.

`circuit_sha256` hashes the finalized constraint matrices and their variable counts. Proving compares the actual circuit's matrix fingerprint with the setup manifest before loading the key. This binds the cryptographic relation without forcing new parameters for a UI edit, package-version change, or source line-ending change. `source_sha256` separately records source and dependency provenance for audit. Neither fingerprint contains private witness assignments. A focused test checks identical fingerprints across different owners and changed fingerprints when the action bound changes.

Proving is separate from rendering and ordinary headless play. The Linux measurement wrapper retains two workers, 4 GiB virtual address space, and 120 seconds. The previous 512-action M2 proof took 5.653 seconds and approximately 1.20 GiB peak resident memory. The authenticated fixed-key user run took 5.858 seconds for proving, 46.416 seconds for key loading/validation, and 54.579 seconds overall, with approximately 1.29 GiB peak RSS. Native binaries use portable Rust APIs; the Bash resource wrapper is optional outside Linux.

## Epochs and payouts

One immutable deployment fixes token name/symbol, start time, epoch duration, epoch count, budget per epoch, and verifier. Start time is deployment block time plus the configured delay; wallet review time cannot expire the configuration. These inputs need user review; test values are not launch tokenomics. The token mints exactly `budget * epochCount` base units into its own escrow. There is no administrator, upgrade, further mint, discretionary withdrawal, or required backend.

Submission chooses the currently open epoch, using chain time. The deadline is exclusive: a transaction mined at or after the end cannot register for that epoch. Offline attempts may predate the epoch. If a proof misses its deadline, its owner may generate a new proof for a later open epoch using the same unsubmitted save and fixed parameters. An already registered identity cannot register again, even for another participant or epoch.

After an epoch ends, its total score is immutable without a separate finalization transaction. A participant directly claims `floor(budget * participantScore / totalScore)` once for that epoch. Multiple distinct expeditions add to the participant's score. Empty epochs pay nothing. Rounding dust and empty-epoch budgets remain locked without rollover. Claims have no expiry after the epoch ends. The combined ERC-20/escrow implementation uses the pinned OpenZeppelin token sources.

## Recorded and optional verification

The user has completed the focused suite, fixed setup, actual saved-game proof, and deployment stages below. The complete two-participant settlement and recovery procedure remains available for later execution under the maintainer's decision; no additional run is requested for launch.

1. Pass `bash scripts/check-rewards.sh`: 26 workspace tests, 18 proof-package tests, seven accounting tests, and eleven Python checks. Accounting tests use an explicit mock; they do not replace real proof tests. The [single smoke-test entry point](SMOKE_TEST.md) combines these with the remaining setup and wallet handoff.
2. Generate candidate fixed parameters with `bash scripts/proof-release.sh setup target/reward-parameters-v1`. Preserve the manifest and public keys. This is the accepted setup operation, not an experimental per-run key.
3. Play a normal private-identity expedition, collect a crystal, seal it, and use the browser-wallet handoff to obtain recipient, contract, and epoch. Practice saves cannot claim.
4. Prove that saved expedition with the fixed parameters, pass its generated real-verifier tests, and register through the user's browser wallet on chain 46630.
5. Complete a two-participant epoch, proportional payouts, an empty epoch, duplicate and wrong-context rejection, and an interrupted-operation recovery check. Preserve public receipts and measurements; never share private saves.

No mainnet deployment, token funding, GitHub publication, or claim of production readiness follows from these commands alone. The user reviews the concrete package and controls signing. Infinite rewarded descent, additional biomes, external randomness, multi-party setup interoperability, and proof aggregation remain deferred.
