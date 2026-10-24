# Robinhood testnet verifier check

Status: a local browser-wallet handoff is prepared for user execution. The helper has not been tested by the assistant or used to submit a transaction. It accepts only experiment packages labeled for testnet chain 46630.

## Start the local page

The generated expedition's successful native run is `target/proof-lab.fsXVTtO3`. From the monorepo:

```sh
bash scripts/check-testnet.sh target/proof-lab.fsXVTtO3
```

This runs six focused helper tests and the generated verifier's four Forge tests. It then starts a standard-library Python HTTP server bound only to `127.0.0.1:8765`. Open `http://127.0.0.1:8765` in the browser containing MetaMask or Rabby. Keep the command running; Ctrl+C stops the local page.

This is a temporary development page on the player's computer, not a project-operated backend, public service, or new game-runtime dependency. No wallet key, password, or seed phrase belongs in a command, file, chat, or the page. The extension handles signing.

## User actions

1. Connect the funded testnet account `0x87660E6337eFb749a3a6DE321F9D38ad587ed8FC`. Approve switching to Robinhood Chain Testnet if requested.
2. Click **Deploy test verifier**. Review and approve this contract-creation transaction in the wallet. It sends zero ETH as value and consumes testnet gas.
3. Wait for the deployed address. Click **Check valid and altered proofs**. These are read-only RPC calls: the valid proof must return true and the altered score must return false.
4. Click **Submit valid verification transaction**. Approve the second zero-value transaction. This supplies an actual testnet receipt and gas measurement.
5. Click **Download result**, then share its JSON content and the Forge output. The file contains public addresses, transaction hashes, receipts, and artifact fingerprints, not signing secrets.

The helper checks the selected chain and account before transactions, pins `chainId` to 46630 in each transaction, and compares deployed runtime bytecode with the compiled local verifier. It cannot switch to mainnet or approve a token allowance. If a receipt remains pending, use **Check pending transaction** instead of sending again. An existing matching verifier address can be pasted into the address field to resume read-only checks.

The valid-proof transaction returning a successful receipt does not itself expose the Solidity Boolean return value. The helper requires the read-only valid/altered checks against the same runtime before submission and records those results separately.

## Inspect without starting the page

```sh
python3 scripts/testnet_check.py target/proof-lab.fsXVTtO3 --prepare-only
```

This creates `testnet-check/index.html` and `testnet-check/package.json` inside that run's directory. All transaction data is available for review before connecting a wallet. The complete helper uses Python's standard library and plain HTML/JavaScript; it fetches no remote JavaScript or wallet SDK.

The exported verifier still uses disposable local experiment parameters and fixture identity/epoch labels. Passing this check establishes testnet compatibility for the measured fixture, not production reward authorization or approval to deploy funded contracts.

Official network configuration: [Robinhood connection documentation](https://docs.robinhood.com/chain/connecting/) and [browser-wallet network setup](https://docs.robinhood.com/chain/add-network-to-wallet/). Testnet chain ID is 46630, the public RPC is `https://rpc.testnet.chain.robinhood.com`, and the explorer is `https://explorer.testnet.chain.robinhood.com`.
