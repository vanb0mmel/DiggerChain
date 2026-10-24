#!/usr/bin/env python3
"""Package fixed public keys and a reviewed player page, excluding all saves."""

import argparse
import hashlib
import json
import re
import zipfile
from pathlib import Path


def package(parameters, label, output):
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,99}", label):
        raise ValueError("Invalid archive label.")
    manifest = json.loads((parameters / "manifest.json").read_text(encoding="utf-8"))
    page_package = json.loads((parameters / "wallet/package.json").read_text(encoding="utf-8"))
    if manifest.get("scheme") != "groth16-bn254-reward-v1" or page_package.get("verifyingKeySha256") != manifest.get("verifying_key_sha256"):
        raise ValueError("The player page and fixed parameters disagree.")
    if not page_package.get("playerMode") or page_package.get("expectedAccount") is not None or page_package.get("parametersPath") != "./parameters":
        raise ValueError("Prepare a player page with --any-account and the reviewed verifier and rewards addresses first.")
    for field in ("verifierAddress", "rewardsAddress"):
        value = page_package.get(field, "")
        if not re.fullmatch(r"0x[0-9a-fA-F]{40}", value) or int(value, 16) == 0:
            raise ValueError("The player page needs reviewed nonzero contract addresses.")
    if page_package.get("chainId") not in (4663, 46630):
        raise ValueError("Unsupported settlement chain.")
    root = Path(__file__).resolve().parents[1]
    expected_page = (root / "contracts/wallet.html").read_text(encoding="utf-8").replace(
        "{{PAYLOAD}}", json.dumps(page_package).replace("<", "\\u003c")
    ).encode()
    page = (parameters / "wallet/index.html").read_bytes()
    if page != expected_page:
        raise ValueError("Player HTML does not match the reviewed template and public package.")
    source = (parameters / "src/Verifier.sol").read_bytes()
    if hashlib.sha256(source).hexdigest() != manifest.get("verifier_source_sha256"):
        raise ValueError("Verifier source fingerprint mismatch.")
    files = {"wallet.html": page, "parameters/src/Verifier.sol": source,
             "parameters/manifest.json": (parameters / "manifest.json").read_bytes()}
    for name, field in [("proving-key.bin", "proving_key_sha256"), ("verifying-key.bin", "verifying_key_sha256")]:
        data = (parameters / name).read_bytes()
        if hashlib.sha256(data).hexdigest() != manifest.get(field):
            raise ValueError(f"Fingerprint mismatch: {name}")
        files[f"parameters/{name}"] = data
    files["SETUP_MODEL.md"] = (root / "docs/SETUP_MODEL.md").read_bytes()
    files["LICENSE-OpenZeppelin.txt"] = (root / "contracts/vendor/openzeppelin/LICENSE").read_bytes()
    files["SHA256SUMS"] = "".join(f"{hashlib.sha256(data).hexdigest()}  {name}\n" for name, data in files.items()).encode()
    output.mkdir(parents=True, exist_ok=True)
    destination = output / f"{label}.zip"
    with zipfile.ZipFile(destination, "x", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in files.items():
            archive.writestr(name, data)
    with destination.with_suffix(".zip.sha256").open("x", encoding="utf-8") as stream:
        stream.write(f"{hashlib.sha256(destination.read_bytes()).hexdigest()}  {destination.name}\n")
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parameters", type=Path)
    parser.add_argument("--label", default="diggerchain-public-parameters-candidate")
    parser.add_argument("--output", type=Path, default=Path("target/dist"))
    args = parser.parse_args()
    try:
        print(package(args.parameters, args.label, args.output))
    except (OSError, ValueError, zipfile.BadZipFile) as error:
        parser.exit(1, f"Cannot package public parameters: {error}\n")


if __name__ == "__main__":
    main()
