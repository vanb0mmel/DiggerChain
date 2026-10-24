# Fixed-parameter gameplay reward check

The maintainer deferred further test execution on 2026-10-22. These steps remain available for optional later verification; they are not launch prerequisites.

This is a user-run check on Robinhood testnet 46630. Do not reuse the old disposable feasibility verifier. The local page uses only a browser wallet and RPC; there is no project-operated backend. Signing keys stay in MetaMask/Rabby. The page handles public proofs and never needs a private save.

For the consolidated one-sitting route, use [docs/SMOKE_TEST.md](../docs/SMOKE_TEST.md) and `bash scripts/smoke-test.sh`. The individual steps below remain available for focused retries.

## 1. Run the focused suite

```sh
bash scripts/check-rewards.sh
```

Expected: 26 workspace tests, 18 proof-package tests, seven contract workflow tests, and eleven Python checks. All must pass. A known earlier SHA-256 test-vector typo has been corrected. The Forge workflow tests use a mock; actual cryptographic checks follow below.

## 2. Create fixed candidate parameters once

```sh
bash scripts/proof-release.sh setup target/reward-parameters-v1
```

The destination must not exist. The command saves public proving/verifying keys, verifier source, and a manifest. It does not save the setup randomness. The active security assumption is an honest single setup operator and machine. Keep these exact public artifacts for all participants in this deployment; do not regenerate them for each run. The script limits the process to two workers, 4 GiB virtual address space, and 120 seconds. A failed or interrupted setup leaves an incomplete directory: choose a new directory for retry and do not deploy an incomplete package.

The finalized constraint-matrix fingerprint binds the key to the proof relation. A change to those matrices rejects the old manifest deliberately; cosmetic and package-version edits do not force replacement parameters. A separate source fingerprint preserves setup provenance. Freeze and validate the final candidate before sharing release parameters.

## 3. Prepare actual saved gameplay

```sh
cargo run --release --locked -- --expedition --save target/reward-player1.json
```

Do not add `--seed`: explicit-seed runs are practice-only. Collect at least one crystal, then press F to seal and Q to exit. Every action is saved. This file contains a private expedition secret: never upload it or include it in a bug report. An agent may drive the same save with `--headless`; spending keys remain outside that interface.

## 4. Open the signing page

```sh
bash scripts/check-reward-wallet.sh target/reward-parameters-v1
```

Open `http://127.0.0.1:8766` in the browser with MetaMask/Rabby. The expected first wallet is `0x87660E6337eFb749a3a6DE321F9D38ad587ed8FC`. The concrete default test configuration is **DiggerChain Test / DCTEST**, eight five-minute epochs, 1,000 test tokens per epoch, and a start one minute after actual deployment. These values do not define launch tokenomics. Every transaction has value 0 ETH and consumes only test gas.

Connect, deploy the fixed verifier, then deploy the combined test token/reward contract. Each deployment requires a wallet confirmation. Record both addresses. The page checks deployed verifier bytecode, reward runtime with compiler-declared immutable locations, and the actual verifier, rules, token metadata, supply, schedule, and budget getters. Refresh after the start if no epoch is open.

Select **Prepare offline proof command**. It displays an exact command for your private save, fixed parameter directory, selected recipient, contract, and currently open epoch. Run it in the monorepo. Proving loads the fixed key and performs no setup. Then run the displayed four Forge tests for that generated real-verifier package. Upload only its public `evm.json` through the page.

Check the real proof and altered-score rejection, then register. The page repeats the check and a read-only registration dry run before requesting the transaction. It records actual gas and public receipts. If the deadline passes, regenerate a proof for a later open epoch from the same unsubmitted save. No gameplay replay or replacement setup is needed.

## 5. Second participant and payouts

Use a second testnet-funded browser wallet and a separate private save:

```sh
cargo run --release --locked -- --expedition --save target/reward-player2.json
bash scripts/check-reward-wallet.sh target/reward-parameters-v1 --account SECOND_WALLET_ADDRESS --save-path target/reward-player2.json --port 8767
```

Replace `SECOND_WALLET_ADDRESS` with its public address. Enter the existing verifier and reward addresses; deploy neither again. Prove and register a second positive score in the same open epoch. Different scores make proportional payouts easier to inspect. After the deadline, select that epoch and claim from each wallet. No finalizer or bot is required. Compare payouts with `floor(1000 * 10^18 * playerScore / totalScore)` base units. Try registering the already used expedition again and claiming twice; both must fail without another successful payout.

Leave one later epoch empty. After its end, it must show no claimable reward and release none of its budget. Rounding dust and unused budgets remain in escrow. Total token supply stays 8,000 test tokens for the default configuration.

The page stores only public addresses, transaction hashes, and receipts in browser recovery storage. Reload during a known pending transaction, reconnect, and check its receipt before submitting anything else. If a wallet transport error returns no transaction hash, inspect wallet activity before retrying. Native proving can be retried into a fresh output directory after interruption; it does not alter the save or parameters.

Download public results from both pages and share their paths and terminal test output. Keep saves private. Successful deployment is not by itself acceptance of the complete reward path; actual registration, payout, negative cases, and recovery still need user confirmation.
