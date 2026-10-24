#!/usr/bin/env python3
"""Prepare a public browser-wallet handoff; default to the testnet candidate."""

import argparse
import hashlib
import json
import re
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path

from testnet_check import bytecode, read_json


REPO = Path(__file__).resolve().parents[1]
ACCOUNT = "0x87660E6337eFb749a3a6DE321F9D38ad587ed8FC"
NAMES = ["rules_version", "seed", "chain_id", "epoch", "expedition_id_high", "score", "charge", "floor", "x", "y", "turns", "expedition_id_low", "recipient", "settlement_contract"]
NETWORKS = {
    46630: {"name": "Robinhood Chain Testnet", "rpc": "https://rpc.testnet.chain.robinhood.com", "explorer": "https://explorer.testnet.chain.robinhood.com"},
    4663: {"name": "Robinhood Chain", "rpc": "https://rpc.mainnet.chain.robinhood.com", "explorer": "https://robinhoodchain.blockscout.com"},
}


def fingerprint(data):
    return hashlib.sha256(data).hexdigest()


def load_package(parameters, account, config, chain=46630):
    if account is not None and (not re.fullmatch(r"0x[0-9a-fA-F]{40}", account) or int(account, 16) == 0):
        raise ValueError("Expected a nonzero 20-byte wallet address.")
    manifest = read_json(parameters / "manifest.json")
    if manifest.get("scheme") != "groth16-bn254-reward-v1" or manifest.get("public_input_names") != NAMES:
        raise ValueError("These are not authenticated v1 reward parameters.")
    if manifest.get("max_actions") != 512 or manifest.get("setup_assumption") != "single_operator_trusted_setup":
        raise ValueError("Unexpected bound or setup model.")
    if (parameters / "verifying-key.bin").stat().st_size > 4096 or (parameters / "src/Verifier.sol").stat().st_size > 65536:
        raise ValueError("Unexpected verifying key or source size.")
    verifying = (parameters / "verifying-key.bin").read_bytes()
    source = (parameters / "src/Verifier.sol").read_bytes()
    if fingerprint(verifying) != manifest.get("verifying_key_sha256") or fingerprint(source) != manifest.get("verifier_source_sha256"):
        raise ValueError("The parameter manifest, verifying key, and source disagree.")
    verifier = read_json(parameters / "out/Verifier.sol/Verifier.json")
    rewards = read_json(REPO / "contracts/out/DiggerChainRewards.sol/DiggerChainRewards.json")
    if verifier.get("methodIdentifiers", {}).get("verify(uint256[8],uint256[14])") != "b11c6afb":
        raise ValueError("Unexpected reward-verifier ABI.")
    if rewards.get("methodIdentifiers", {}).get("register(uint256[8],uint256[14])") != "be706fea":
        raise ValueError("Unexpected settlement ABI.")
    reward_runtime = bytecode(rewards["deployedBytecode"]["object"], 24576)
    ranges = [entry for entries in rewards["deployedBytecode"]["immutableReferences"].values() for entry in entries]
    for entry in ranges:
        if entry.get("length") != 32 or not 0 <= entry.get("start", -1) <= (len(reward_runtime) - 2) // 2 - 32:
            raise ValueError("Invalid immutable bytecode reference.")
    return {
        "chainId": chain, "network": NETWORKS[chain], "expectedAccount": account, "names": NAMES, "config": config,
        "verifyingKeySha256": manifest["verifying_key_sha256"],
        "manifestSha256": fingerprint((parameters / "manifest.json").read_bytes()),
        "verifierCreation": bytecode(verifier["bytecode"]["object"], 49152),
        "verifierRuntime": bytecode(verifier["deployedBytecode"]["object"], 24576),
        "rewardsCreation": bytecode(rewards["bytecode"]["object"], 49152),
        "rewardsRuntime": reward_runtime, "immutableRanges": ranges,
        "methods": rewards["methodIdentifiers"],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parameters", type=Path)
    parser.add_argument("--account", default=ACCOUNT)
    parser.add_argument("--any-account", action="store_true", help="Let participants connect their own selected browser wallet.")
    parser.add_argument("--chain", type=int, choices=NETWORKS, default=46630)
    parser.add_argument("--port", type=int, default=8766)
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--name", default="DiggerChain Test")
    parser.add_argument("--symbol", default="DCTEST")
    parser.add_argument("--duration", type=int, default=300, help="Test epoch duration in seconds.")
    parser.add_argument("--epochs", type=int, default=8)
    parser.add_argument("--budget", type=int, default=1000, help="Whole test tokens per epoch (18 decimals).")
    parser.add_argument("--start-delay", type=int, default=60)
    parser.add_argument("--save-path", help="Private save path used only in displayed commands; {account} expands to the connected public address. This helper never reads the save.")
    parser.add_argument("--verifier-address", help="Preconfigure an existing verifier.")
    parser.add_argument("--rewards-address", help="Preconfigure an existing rewards contract; hides deployment controls when both addresses are supplied.")
    args = parser.parse_args()
    if not 1 <= args.duration < 2**64 or not 1 <= args.epochs < 2**64 or not 1 <= args.budget * 10**18 < 2**128:
        parser.error("Test duration, count, or budget is outside the contract range.")
    if not 0 <= args.start_delay <= 86400 or not 1 <= len(args.name.encode()) <= 64 or not 1 <= len(args.symbol.encode()) <= 16:
        parser.error("Provide a name, symbol, and start delay between 0 and 86400 seconds.")
    config = {"name": args.name, "symbol": args.symbol, "duration": str(args.duration), "count": str(args.epochs),
              "budget": str(args.budget * 10**18), "startDelay": str(args.start_delay)}
    for address in (args.verifier_address, args.rewards_address):
        if address is not None and (not re.fullmatch(r"0x[0-9a-fA-F]{40}", address) or int(address, 16) == 0):
            parser.error("Existing contracts require nonzero 20-byte addresses.")
    if args.rewards_address and not args.verifier_address:
        parser.error("Supply the matching verifier address with the rewards address.")
    try:
        parameters = args.parameters.resolve()
        payload = load_package(parameters, None if args.any_account else args.account, config, args.chain)
        payload["verifierAddress"] = args.verifier_address
        payload["rewardsAddress"] = args.rewards_address
        payload["playerMode"] = bool(args.verifier_address and args.rewards_address)
        payload["parametersPath"] = "./parameters" if payload["playerMode"] else str(parameters)
        payload["savePath"] = args.save_path or ("./saves/{account}.json" if payload["playerMode"] else "target/reward-player1.json")
        template = (REPO / "contracts/wallet.html").read_text(encoding="utf-8")
        page = template.replace("{{PAYLOAD}}", json.dumps(payload).replace("<", "\\u003c")).encode("utf-8")
        bundle = parameters / "wallet"
        bundle.mkdir(exist_ok=True)
        (bundle / "index.html").write_bytes(page)
        (bundle / "package.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    except (OSError, KeyError, TypeError, ValueError) as error:
        parser.exit(1, f"Cannot prepare wallet handoff: {error}\n")
    print(f"Prepared: {bundle}", flush=True)
    print(f"Network: {args.chain}. Wallet: {payload['expectedAccount'] or 'selected by each participant'}. Public key: {payload['verifyingKeySha256']}", flush=True)
    print(f"Test configuration: {args.epochs} epochs, {args.duration} seconds each, {args.budget} {args.symbol} per epoch.", flush=True)
    if args.prepare_only:
        return

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path not in ("/", "/index.html"):
                self.send_error(404)
                return
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(page)))
            self.send_header("Cache-Control", "no-store")
            self.send_header("X-Content-Type-Options", "nosniff")
            self.send_header("Content-Security-Policy", "base-uri 'none'; frame-ancestors 'none'")
            self.end_headers()
            self.wfile.write(page)

    try:
        with HTTPServer(("127.0.0.1", args.port), Handler) as server:
            print(f"Open http://127.0.0.1:{args.port}. All transactions require a button click and browser-wallet approval.", flush=True)
            server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
