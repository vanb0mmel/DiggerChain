# One-sitting candidate smoke test

Status as of 2026-10-22: the maintainer deferred further tests to accelerate launch. This guide is retained for optional future verification, not as a launch prerequisite or a request to rerun setup. Existing results are preserved in the [partial smoke record](../contracts/measurements/2026-10-21-smoke/README.md).

The assistant prepared the complete candidate and static checks. The user executes this test and owns acceptance. It uses Robinhood **testnet 46630**, never mainnet. The first script runs tests, builds executables, performs the accepted fixed candidate setup once, compiles its verifier, and opens a localhost page. It does not sign or send transactions automatically.

## Start once

From the monorepo:

```sh
bash scripts/smoke-test.sh
```

Keep that terminal open. It prints `Smoke-test artifacts: .../target/smoke.XXXXXXXX`. Logs, fixed public parameters, and the generated page stay in that directory. The private player saves will also live there, with mode 0600 on Linux. Share logs and public receipts only; never share `player-*.json`.

Expected checks: **26 workspace tests, 18 proof-package tests, seven Solidity workflow tests, and eleven Python checks**, all passing. The known earlier SHA-256 expected-value typo is fixed. Proof-constraint rejection tests may print `ConstraintLayer` notices for intentionally invalid witnesses. Foundry may also flag intentional chain-time deadline comparisons and casts already bounded by the explicit score/deadline guards; these are lint notices, not compilation failures. Test compilation can download missing locked Cargo dependencies; repeated builds reuse the cache. GNU time is unnecessary.

If any stage fails, stop and share `checks.log`, `setup.log`, or the last error plus the artifact path. Do not delete the directory or rerun setup over it. A repeat of the full script creates a new directory and never overwrites an existing expedition or parameter set. It is a new candidate deployment, not a per-player setup process.

## Check the game before deploying

In a second terminal, use an isolated practice save:

```sh
cargo run --release --locked -- --expedition --new --seed 0 --save target/visual-smoke.json
```

Check movement and bump-to-dig, flask use with E, crystal collection, both portal directions, and G for the alternate view. Changed rock and items must survive revisits. Quit with Q, then resume without `--new --seed 0`. Position, score, charge, and discovered map must survive. Resize the terminal and exit; normal terminal input must return. Press F when finished; resuming that save must reject further game actions.

Open the preserved authored prototype explicitly:

```sh
cargo run --release --locked -- --prototype
```

The practice save cannot earn rewards. Automated route and save tests cover the same simulation and headless protocol; this manual part focuses on behavior and appearance. No screenshot loop is required.

## Deploy the candidate on testnet

Open `http://127.0.0.1:8766` with MetaMask or Rabby. Stop any older server on that port first. Confirm the page says **46630**. Defaults are DiggerChain Test / DCTEST, eight five-minute epochs, and 1,000 test tokens per epoch. These are test values, not launch tokenomics.

Connect the funded first wallet, then deploy the verifier and token/rewards contract using buttons 2 and 3. Review both wallet confirmations. Transaction value is 0 ETH; test gas is spent. The page pins the fixed verifier, checks reward code and configuration, and remembers the public addresses. Do not deploy another pair for the second participant. Refresh after the displayed start time if needed.

## Play, prove, and register two participants

Click **Prepare offline proof command**. Before the first epoch starts, the page explains the waiting state without treating it as a failure. The page displays commands for the connected wallet's separate private save. In the second terminal, run the displayed game command, collect at least one crystal, press F, and quit. **After playing, prepare the command again** to select a fresh open epoch. If less than two minutes remain, prefer the next epoch. The displayed time remaining is a snapshot from the last refresh, not a live timer. Then run the displayed proof command and the four real-verifier Forge tests. Proof generation uses the fixed setup from this smoke run; it never creates new parameters. The wrapper keeps two proof workers, 4 GiB virtual address space, and a 120-second timeout.

Load only the proof output's public `evm.json`. The check button performs only read-only checks. **Button 6 requests the registration transaction; approve it and wait for `registration confirmed` and the registered score/epoch message.** A valid proof must return true, the altered score must return false, and registration must succeed. The page records actual gas and receipts. If the epoch closes before registration, prepare a new context and regenerate only the proof for the next open epoch from the same unsubmitted save.

Switch to a second funded testnet account in the browser wallet and click **Connect wallet** again. The page works with either participant and gives each account a different save path. Repeat the same play/prove/check/register path in the **same open epoch**, using the same contracts and keys. Aim for different scores, such as one and two crystals on the first floor, so proportional rewards are easy to compare. Native proof checks cover every public input, including the recipient and domain.

## Claim and reject repeats

After that epoch ends, enter its number and claim from each wallet, reconnecting after switching accounts. Expected payout in base units:

```text
floor(1000 * 10^18 * participantScore / totalScore)
```

The contract closes registration automatically at its deadline. There is no finalize button or closing bot. Claims do not change token supply. Both payouts together must not exceed the epoch budget. The downloaded result includes each claim's score, total score, amount, and receipt.

Try the already registered proof again and repeat a claim. Read-only checks or gas estimation must reject these before another successful transaction. Leave one later epoch without registrations. After its end, select it and try claim: the reported claimable amount must be zero, and the page must explain that the epoch is empty without requesting a transaction. A wallet with no registered score or an already completed claim must also receive a specific explanation before gas estimation. A legitimate zero payout caused by proportional rounding is still claimable once. Unused budgets and rounding dust stay locked; they never increase another epoch's budget.

## Recovery and handoff

Before one registration or claim, enable **Recovery check**. After the wallet returns its transaction hash, the page deliberately leaves receipt processing for later. Reload, reconnect, and use **Check pending transaction**. It must recover the saved hash and actual receipt rather than submit a duplicate, even if the chain confirmed it before the reload. This makes the check practical on a fast chain. If the wallet returns a transport error without a hash, inspect wallet activity before retrying. For an interrupted proof, retry into a fresh output directory; the sealed save and fixed keys remain unchanged.

Click **Download public result and receipts**. Share its path, the smoke artifact path, and concise gameplay findings. The assistant can inspect those local files; do not upload private saves. The automated checks log is already captured in the smoke directory.

Passing this scenario would validate the local Linux candidate and the actual testnet reward path. The remaining runtime checks are currently deferred. Native platform artifacts, launch token configuration, the GitHub destination, concrete deployment review, and user-controlled publication remain separate deliverables. They are not silently treated as completed by this test.

## Continue the recorded partial smoke

The [2026-10-21 record](../contracts/measurements/2026-10-21-smoke/README.md) passed all 62 checks, fixed setup, saved-game proving, and both deployments, but missed registration. Do not rerun the full script or create replacement keys. The existing private save can be proved again without replaying the game. The assistant refreshed this run's generated HTML with the clarified wallet UI.

Stop the old localhost server with Ctrl+C, then reopen the updated page from the monorepo:

```sh
target/release/diggerchain --wallet target/smoke.r5AqRUzR/parameters/wallet/index.html
```

Reconnect at `http://127.0.0.1:8766`. The browser should recover the existing addresses; both are also listed in the measurement record. Refresh, select an epoch with enough time remaining, prepare a new command, skip the already finished game step, and prove the same save. Load the new public `evm.json`, click **6. Register score**, approve, and wait for confirmation. After that epoch ends, claim its number. The earlier epoch 1 was empty; selecting it cannot pay this new registration. The two-participant and recovery steps above remain available for the deferred verification.

This deployment accepts submissions only until **2026-10-21 19:15:40 UTC (22:15:40 Moscow time)**. If its eight epochs have ended, reuse the same parameters and verifier, but prepare a longer test schedule:

```sh
bash scripts/check-reward-wallet.sh target/smoke.r5AqRUzR/parameters --any-account --duration 900 --save-path "$PWD/target/smoke.r5AqRUzR/player-{account}.json"
```

Stop the previous server first. Keep the existing verifier address, skip button 2, and deploy **only a new token/rewards contract with button 3**. This is an explicit browser-wallet transaction on testnet. Its epochs last 15 minutes, allowing both participants more time; the budget and count are unchanged. Prepare and generate fresh proofs for the new contract address. Do not upload the old contract-bound proof or rerun setup. All checks and payouts must refer to this new deployment.
