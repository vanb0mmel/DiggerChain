#!/usr/bin/env python3
"""Serve a local, wallet-signed, experiment-only Robinhood testnet check."""

import argparse
import hashlib
import json
import re
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path


CHAIN_ID = 46630
SIGNATURE = "verify(uint256[8],uint256[11])"
SELECTOR = "417ed909"
SCALAR_FIELD = 21888242871839275222246405745257275088548364400416034343698204186575808495617
BASE_FIELD = 21888242871839275222246405745257275088696311157297823662689037894645226208583
DEFAULT_ACCOUNT = "0x87660E6337eFb749a3a6DE321F9D38ad587ed8FC"


def read_json(path):
    if path.stat().st_size > 4 * 1024 * 1024:
        raise ValueError(f"Artifact is unexpectedly large: {path.name}")
    return json.loads(path.read_text(encoding="utf-8"))


def word(value, modulus):
    if not isinstance(value, str) or not re.fullmatch(r"[0-9]{1,78}", value):
        raise ValueError("Expected a canonical unsigned decimal word.")
    number = int(value)
    if not 0 <= number < modulus:
        raise ValueError("A proof coordinate or public scalar is outside its field.")
    return number


def calldata(proof, inputs):
    if not isinstance(proof, list) or not isinstance(inputs, list) or len(proof) != 8 or len(inputs) != 11:
        raise ValueError("Expected eight proof coordinates and eleven public inputs.")
    words = [word(value, BASE_FIELD) for value in proof]
    words += [word(value, SCALAR_FIELD) for value in inputs]
    return "0x" + SELECTOR + "".join(f"{value:064x}" for value in words)


def bytecode(value, maximum):
    if not isinstance(value, str) or not re.fullmatch(r"0x(?:[0-9a-fA-F]{2})+", value):
        raise ValueError("Missing or unlinked compiled verifier bytecode. Run the Forge check first.")
    if (len(value) - 2) // 2 > maximum:
        raise ValueError("Compiled bytecode exceeds the supported contract size.")
    return value.lower()


def load_package(directory, account):
    if not re.fullmatch(r"0x[0-9a-fA-F]{40}", account):
        raise ValueError("The expected wallet must be a 20-byte EVM address.")
    report = read_json(directory / "report.json")
    evm = read_json(directory / "evm.json")
    artifact = read_json(directory / "out/Verifier.sol/Verifier.json")
    if report.get("native_verification") != "passed" or report.get("cryptographic_proof") is not True:
        raise ValueError("This package has no passing native cryptographic proof.")
    if report.get("production_ready") is not False:
        raise ValueError("This helper is exclusively for disposable experiment verifiers.")
    if evm.get("function") != SIGNATURE or artifact.get("methodIdentifiers", {}).get(SIGNATURE) != SELECTOR:
        raise ValueError("Unexpected verifier ABI or function selector.")
    proof, inputs = evm["proof"], evm["public_inputs"]
    valid = calldata(proof, inputs)
    if inputs != report.get("public_inputs_decimal"):
        raise ValueError("The native report and EVM public inputs disagree.")
    if word(inputs[2], SCALAR_FIELD) != CHAIN_ID:
        raise ValueError("The proof is not labeled for Robinhood testnet 46630.")
    altered = list(inputs)
    altered[5] = str((int(altered[5]) + 1) % SCALAR_FIELD)
    creation = bytecode(artifact["bytecode"]["object"], 49152)
    runtime = bytecode(artifact["deployedBytecode"]["object"], 24576)
    return {
        "chainId": CHAIN_ID,
        "expectedAccount": account,
        "experiment": report["experiment"],
        "run": directory.name,
        "proofSha256": hashlib.sha256((directory / "proof.bin").read_bytes()).hexdigest(),
        "runtimeSha256": hashlib.sha256(bytes.fromhex(runtime[2:])).hexdigest(),
        "creation": creation,
        "runtime": runtime,
        "validData": valid,
        "alteredData": calldata(proof, altered),
        "publicInputs": inputs,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("run_directory", type=Path)
    parser.add_argument("--account", default=DEFAULT_ACCOUNT)
    parser.add_argument("--port", default=8765, type=int)
    parser.add_argument("--prepare-only", action="store_true", help="Write a reviewable local package without starting a server.")
    args = parser.parse_args()
    directory = args.run_directory.resolve()
    try:
        payload = load_package(directory, args.account)
    except (OSError, KeyError, TypeError, ValueError) as error:
        parser.exit(1, f"Cannot prepare the testnet check: {error}\n")
    template = Path(__file__).resolve().parents[1] / "experiments/proof-lab/testnet.html"
    page = template.read_text(encoding="utf-8").replace(
        "{{PAYLOAD}}", json.dumps(payload).replace("<", "\\u003c")
    ).encode("utf-8")
    bundle = directory / "testnet-check"
    bundle.mkdir(exist_ok=True)
    (bundle / "index.html").write_bytes(page)
    (bundle / "package.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Prepared: {bundle}", flush=True)
    print(f"Network: Robinhood Chain Testnet ({CHAIN_ID}); expected wallet: {args.account}", flush=True)
    print(f"Runtime SHA-256: {payload['runtimeSha256']}", flush=True)
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
            print(f"Open http://127.0.0.1:{args.port} in the browser with MetaMask or Rabby.", flush=True)
            print("Signing happens only after a button click and wallet approval. Ctrl+C stops this local page.", flush=True)
            server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
