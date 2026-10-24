# Partial reward smoke result, 2026-10-21

Status: automated checks, fixed setup, saved-game proving, and deployment passed. **Score registration and payout did not complete.** This record does not close the reward-flow acceptance gate.

The user supplied `diggerchain-rewards-46630-smoke.json`. The associated local artifacts were `target/smoke.r5AqRUzR` and `target/reward-proof-1-1792608173449`. The files here contain only public results, proof data, parameter metadata, and test output; no private save, expedition secret, spending key, or setup randomness was copied. `SHA256SUMS` records these snapshots.

## Passed checks

- 26 workspace tests, 18 proof-package tests, seven Solidity accounting tests, and eleven Python checks: **62 passed** in the user's captured execution. The accounting tests use a mock verifier.
- Dedicated fixed candidate setup completed in 14.276 seconds. Public proving key: 93,568,400 bytes. Its verifying-key fingerprint and manifest fingerprint match the wallet handoff.
- An actual saved expedition generated a proof: score 1, 11 successful actions, 484,351 constraints, all 14 altered public inputs rejected, and native verification passed.
- Proving took 5.858 seconds; key loading and validation took 46.416 seconds; the full process took 54.579 seconds. Peak RSS was 1,356,364 KiB, approximately 1.29 GiB. This is the authenticated fixed-key workflow, distinct from the earlier disposable M2 setup benchmark.
- The browser report records real-proof acceptance, altered-score rejection, and a successful registration dry run. These are read-only checks, not a submitted registration.
- Both deployment receipts have status `0x1`. Read-only RPC inspection independently confirmed their receipts, matching verifier/runtime code, schedule, supply, and escrow balance.

## Transactions and chain state

Network: Robinhood Chain testnet, chain ID 46630.

| Operation | Address | Transaction | Gas used |
| --- | --- | --- | ---: |
| Verifier deployment | `0x45f4e6c42e8a71d495d5f23793ad949a081d7364` | `0xfc375c612f7a3dcc202980ea28b21abd2a5fb5c2c781c8cd813a71a348b0f222` | 704,598 |
| Token/rewards deployment | `0x7fb642e5afc33696daf0481401f0927abec309ad` | `0x8d69add2f233329b90fa168a73926a7994efe108b23fc23a3c91da11e6169124` | 1,150,537 |

Both transactions used an effective gas price of 10,000,000 wei. Their combined fee was 0.00001855135 test ETH. The contract minted 8,000 DCTEST into its own escrow; this was issuance, not a player payout.

The read-only snapshot at 18:52:54 UTC recorded zero total score in all eight epochs, zero player score in epoch 1, zero player token balance, an unused expedition ID, and the full supply still in escrow. See `rpc-review.json`. This is a historical snapshot, not a claim about later chain activity.

## Why the reward path stopped

The deployment used eight five-minute epochs, starting at 18:35:40 UTC. The uploaded proof targeted epoch 1, open from **18:40:40 to 18:45:40 UTC**, with an exclusive deadline. The attempted registration check at 18:45:47.307 UTC was approximately seven seconds late, so the page correctly refused it. No registration transaction appears in the report or the inspected chain state.

At 18:45:58 UTC the user attempted to claim epoch 1. Its total and player scores were both zero, so the claim reverted during gas estimation and no claim transaction was sent. This is consistent with the contract's empty-epoch rule. The earlier pre-start message was an ordinary waiting state.

## Follow-up

The wallet UI now distinguishes checking from confirmed registration, shows time remaining at refresh, instructs the player to refresh the proof context after offline play, treats the pre-start state as waiting, and explains empty, unregistered, or already-claimed rewards before gas estimation. These UI changes still require user verification; the 62-test result predates them.

The same unsubmitted save and public parameters remain reusable. This deployment's final submission deadline is **19:15:40 UTC on 2026-10-21**. A later open epoch requires a fresh proof context and proof, not new gameplay or setup. After all epochs end, only a new matching rewards deployment is needed; its contract address must be included in a newly generated proof. The verifier and setup can remain unchanged.

Still required: actual registration and claim receipts, a two-participant proportional payout, repeated-submission/payout checks through the wallet flow, and transaction recovery. No production freeze, mainnet readiness, or release acceptance is inferred from this partial result.
