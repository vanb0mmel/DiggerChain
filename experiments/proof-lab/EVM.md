# First local EVM verification

Status: the user compiled the verifier from `target/proof-lab.psxh3L7W` and passed all four local EVM tests. The valid verifier call measured 269,032 gas, excluding deployment and a complete reward transaction. Robinhood testnet verification remains unperformed. No proof regeneration or game-code change was required.

Installation checked: `forge`, `cast`, `anvil`, and `chisel` 1.8.5 and `foundryup` 0.0.8 all launch successfully with `--version`. Solidity 0.8.28 also launches from `target/solc-0.8.28` after a verified download. The assistant checked installation and configuration only; the user compiled Solidity and ran the EVM tests.

## Install the required tool

Foundry's `forge` runs the generated Solidity tests inside a local EVM. It does not need a wallet, testnet ETH, a running RPC node, or a blockchain transaction for these tests. Foundry and the pinned Solidity compiler are separate development tools, not game-runtime dependencies.

Foundry is installed under `$HOME/.foundry` on the selected laptop; no reinstall is needed. For a fresh setup, the official [foundryup installation instructions](https://github.com/foundry-rs/foundryup) use the bootstrap installer below. Run these commands from the repository directory. They download development tools and require no `sudo`:

```sh
mkdir -p target
curl -fsSL https://raw.githubusercontent.com/foundry-rs/foundryup/HEAD/foundryup-init.sh -o target/foundryup-init.sh
env FOUNDRY_DIR="$HOME/.foundry" sh target/foundryup-init.sh
env FOUNDRY_DIR="$HOME/.foundry" "$HOME/.foundry/bin/foundryup"
"$HOME/.foundry/bin/forge" --version
```

The explicit installation directory and executable path make a persistent PATH edit unnecessary. The assistant has not run this installer or changed shell configuration. The bootstrap downloads `foundryup`; running it downloads the Foundry release tools. Keep its normal release verification enabled. Report any installation failure before continuing.

## Check the existing proof

The first automatic compiler download from `binaries.soliditylang.org` timed out before compilation. The same pinned Linux compiler was downloaded from the official [solc-bin repository](https://github.com/argotorg/solc-bin) into `target/solc-0.8.28`. Its SHA-256 matches the [official compiler index](https://raw.githubusercontent.com/argotorg/solc-bin/gh-pages/linux-amd64/list.json):

```text
Version: 0.8.28+commit.7893614a.Linux.g++
Size: 15708640 bytes
SHA-256: 9a0fb7e0db2c0641dbae1c5cc645dc686820c83af516226abb1c0a2f76636f25
```

From the same repository directory, select that compiler explicitly and disable network access:

```sh
"$HOME/.foundry/bin/forge" test --root target/proof-lab.psxh3L7W --use "$PWD/target/solc-0.8.28" --offline -vv
```

This command uses the existing compiler without downloading it again. The binary is a generated local tool under the ignored `target` directory; deleting it requires downloading it again. On a fresh setup, omitting `--use` and `--offline` lets Forge download the Solidity 0.8.28 compiler pinned by this run's `foundry.toml`. The command compiles this existing verifier package; it does not rerun Rust proving. The Rust proof wrapper's resource limits do not automatically apply to Forge or compiler processes.

Expected: four passing tests:

1. The original proof verifies, and `verifier_call_gas_excluding_deployment` is printed.
2. Each of the eleven altered public inputs is rejected.
3. A public scalar outside the canonical field range is rejected.
4. A proof coordinate outside the canonical field range is rejected.

The printed gas value measures a local verifier call, excluding deployment and a complete reward transaction. It is not a Robinhood Chain fee. A passing result closes the local EVM compatibility check only; Robinhood testnet behavior, longer-route measurements, and production setup remain open.

## Recorded user result

The user supplied a successful run using the installed Foundry 1.8.5 and Solidity 0.8.28. Compilation of two files took 312.49 ms. All four tests passed, with zero failures or skips; the complete suite run took 28.41 ms. The original proof was accepted, each of eleven altered public inputs was rejected, and the two noncanonical-value cases were rejected.

The emitted `verifier_call_gas_excluding_deployment` was 269,032 gas. The separate 275,222-gas test total includes test work beyond that measured call. These are local execution measurements, not elapsed proving time, a live-chain transaction receipt, or a token-denominated fee.

The [user-supplied output](measurements/evm-seed0-steps64.txt) and [artifact fingerprints](measurements/evm-seed0-steps64.sources.sha256) are preserved outside the ignored build directory. The native `report.json` remains unchanged: its `evm_verification: "not_run"` records the earlier native process, before this separate EVM check. No deployment or transaction was submitted to a live blockchain.

The verifier key was generated in a local single-party experiment. This package must not be used for funded rewards. The wallet prepared for Robinhood testnet will be used in a later explicitly reviewed on-chain step.
