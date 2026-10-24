# Bounded expedition rewards

Status: implementation for user validation, not a deployed economy. The real verifier is generated from the fixed authenticated circuit setup; the old feasibility verifier cannot authorize this contract.

`DiggerChainRewards` combines the ERC-20 token and immutable epoch escrow. It mints exactly `epochBudget * epochCount` base units to itself at construction. There is no owner, upgrade mechanism, discretionary mint, withdrawal, separate finalization transaction, or required closing service. Token name, symbol, schedule, and budget remain deployment inputs for user review.

The constructor accepts a start delay in seconds and fixes `startsAt` to deployment block time plus that delay. Time spent reviewing the wallet confirmation cannot invalidate an absolute start timestamp. Duration, count, and the computed final deadline are range-checked before the start is narrowed to 64 bits.

During an epoch, an expedition proof registers a positive score for its bound recipient. The sender must be that recipient. Rules, chain ID, contract address, epoch deadline, and a globally single-use expedition identity are checked before credit. A participant may register multiple different expeditions. Each expedition can be used once across all epochs of this settlement contract.

After the deadline, anyone can claim their own share directly: `floor(epochBudget * playerScore / totalScore)`. A payout can be claimed once per player and epoch. Empty-epoch funds and rounding dust remain in escrow without rollover or withdrawal. The fixed budget bounds issuance; it does not establish token value or equal earnings. Submission time assigns the epoch; the proof does not establish when the game was played.

The fixed setup trust assumption is documented in [SETUP_MODEL.md](../docs/SETUP_MODEL.md). Replacing that process later requires describing the actual new process and deploying the appropriate approved verifier; the existing immutable contract cannot silently adopt it.

## Dependencies

The ERC-20 implementation is the minimal transitive source subset of [OpenZeppelin Contracts v5.4.0](https://github.com/OpenZeppelin/openzeppelin-contracts/tree/v5.4.0), vendored unmodified under `vendor/openzeppelin` with its MIT license and source fingerprints. Foundry installs no additional libraries. Use Solidity 0.8.28, as in the measured proof experiment.

## User checks

From the monorepo, using the previously downloaded compiler:

```sh
"$HOME/.foundry/bin/forge" test --root contracts --use "$PWD/target/solc-0.8.28" --offline -vv
```

Seven workflow tests use an explicitly named mock verifier to isolate accounting, deadlines, authorization, duplicates, empty epochs, rounding, and basic token transfer. The generated proof package supplies separate real-verifier tests. Both are required; mocked accounting tests alone do not establish proof security. The user then validates actual offline gameplay, registration, and two-participant payouts on Robinhood testnet before any release or funding.
