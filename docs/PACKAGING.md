# Candidate artifacts and release preparation

For the public alpha, package only the game with `python3 scripts/package_release.py --game target/release/diggerchain --label diggerchain-0.1.0-alpha.1-linux-x86_64-local`. The default instructions are `ALPHA_INSTALL.md`; `--proof` explicitly opts into the separate development bundle. The workflow builds Linux, Windows, and both macOS architectures without running tests. On an alpha tag, it publishes the four build artifacts as a prerelease. Earlier proof-package details follow as development history.

Status: scripts and manual CI workflow are prepared. The assistant builds the local Linux candidate and performs static checks; the user owns execution of tests and platform smoke checks. No remote repository, hosted workflow run, public release, or mainnet deployment is configured by this file.

## Published alpha, 2026-10-24

The public [alpha prerelease](https://github.com/vanb0mmel/DiggerChain/releases/tag/v0.1.0-alpha.1) includes Linux x86-64, Windows x86-64, macOS Apple silicon, and macOS Intel archives with checksums. The recommended Windows file ends in `-portable.zip` and statically includes its C runtime. Build and package jobs passed; additional platform runtime tests remain deferred. Exact sizes, hashes, and build runs are recorded in [ALPHA_ARTIFACTS.json](ALPHA_ARTIFACTS.json). No private draft history, saves, setup material, or render caches were published.

## Local candidate, 2026-10-22

The human-delver build is `target/dist/diggerchain-candidate-linux-local-human.zip` (3,010,547 bytes), with SHA-256 `6bb369a65c711712a3622bd38764bbee34fec015883e736d23f6ba2e923cbf5a`. It contains the Linux game, separate proof executable, current installation instructions, dependency notices, and internal checksums. It is a local-host build, not a Windows/macOS build or published release.

Workspace formatting, Clippy with warnings denied, and release compilation of both executables completed. No gameplay, test suite, proving, setup, browser, or signing operation was executed during this update. The reward source fingerprint still matches the user's fixed setup; the human presentation and default-mode change require no replacement keys. Normal launch selects the saved expedition; the authored scene uses `--prototype`.

## Native executable archive

After compiling both binaries:

```sh
python scripts/package_release.py \
  --game target/release/diggerchain \
  --proof experiments/proof-lab/target/release/diggerchain-proof \
  --label diggerchain-candidate-linux-local
```

The archive includes only those explicitly named native executables, install instructions, dependency license notices, and checksums. Private saves and parameters are never discovered by directory traversal. The source notice collector reads Cargo's compiled-target dependency metadata and copies upstream license files, plus toolchain copyright notices when installed. `INDEX.json` preserves declared license, author, and repository information, including packages whose upstream crate omitted a separate license file. Registry licenses remain unmodified.

`.github/workflows/native-candidates.yml` prepares the corresponding archives on Linux, Windows, and macOS once a GitHub repository is provided. It uploads build artifacts; alpha tags also publish a prerelease after all target builds pass. It does not generate proof parameters. The optional checks workflow is also manual, following the maintainer's 2026-10-22 decision to defer further tests. It remains available for the shared simulation, ownership constraints, and artifact parsers. Platform builds and deferred runtime checks are recorded separately.

## Public parameter and wallet bundle

After selecting the fixed parameters and concrete deployment configuration, prepare a player page with the reviewed addresses and exact settings:

```sh
python scripts/reward_wallet.py PARAMETERS_DIRECTORY --prepare-only --any-account \
  --chain 46630 --verifier-address REVIEWED_VERIFIER_ADDRESS \
  --rewards-address REVIEWED_REWARDS_ADDRESS
python scripts/package_parameters.py PARAMETERS_DIRECTORY
```

Replace the uppercase placeholders. Supply the actual `--name`, `--symbol`, `--duration`, `--epochs`, and `--budget` values if they differ from the test defaults. Chain 4663 is supported explicitly for a separately reviewed mainnet package; the consolidated smoke script always selects 46630. [Official Robinhood network configuration](https://docs.robinhood.com/chain/add-network-to-wallet/) supplies the chain IDs, public RPCs, and explorers. No mainnet package is approved by preparing these scripts.

The bundle validates key and verifier-source fingerprints, requires a player page that accepts participants' own wallets, pins nonzero contract addresses, and verifies that HTML matches the repository template and public payload. It includes only public proving/verifying keys, their manifest, verifier source, the player wallet page, the active setup assumptions, the OpenZeppelin license, and checksums. It excludes player saves and operator deployment controls. No setup randomness belongs in either archive.

Extract the public bundle into the executable directory. The native game serves `wallet.html` with `--wallet`; ordinary players need no compiler, Python, Node.js, Foundry, backend account, or embedded wallet. Their proof tool uses `./parameters`, and each connected wallet gets a separate local save path. The page displays shell-specific native commands. Wallet spending keys stay in the browser wallet; private expedition history stays in the local file and prover.

## Before actual publication

Record the existing [smoke result](../contracts/measurements/2026-10-21-smoke/README.md), public receipts, measured proof time and memory, actual native builds, deferred checks, token configuration, verifier and parameter fingerprints, and artifact checksums. The user deferred additional tests; the [optional procedure](SMOKE_TEST.md) is retained without treating unrun stages as passed. Review the immutable epoch policy and current setup trust assumption against the deployed code. Receive the user-provided GitHub destination and approval of the concrete publication/deployment package. The user controls signing and funds.
