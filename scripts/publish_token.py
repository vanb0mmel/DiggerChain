#!/usr/bin/env python3
"""Publish supplied token details to the website and GitHub using existing builds."""

import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import urllib.request

from website_content import validate_config

ROOT = Path(__file__).resolve().parents[1]
START = "<!-- TOKEN_INFO_START -->"
END = "<!-- TOKEN_INFO_END -->"
FILES = ["README.md", "website/site.json"]


def update_readme(text, config):
    if text.count(START) != 1 or text.count(END) != 1:
        raise ValueError("README must contain exactly one token-information block.")
    before, remainder = text.split(START)
    _, after = remainder.split(END)
    token = config["token"]["contractAddress"]
    links = config["links"]
    block = (f"\n## $DCG on Robinhood Chain\n\n"
             f"Chain ID: **4663** · Contract address: `{token}`\n\n"
             f'[X / Twitter](<{links["x"]}>) · [PonsFamily](<{links["ponsFamily"]}>)\n')
    return before + START + block + END + after


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ca", required=True)
    parser.add_argument("--x", required=True)
    parser.add_argument("--pons", required=True)
    args = parser.parse_args()
    started = time.monotonic()
    fly = Path.home() / ".fly/bin/flyctl"
    if not fly.is_file():
        parser.error("The prepared Fly CLI is missing.")
    original = {name: (ROOT / name).read_text(encoding="utf-8") for name in FILES}
    config = json.loads(original["website/site.json"])
    config["token"]["contractAddress"] = args.ca
    config["links"].update(x=args.x, ponsFamily=args.pons)
    validate_config(config)
    readme = update_readme(original["README.md"], config)
    for options in ([], ["--cached"]):
        subprocess.run(["git", "diff", *options, "--quiet", "--", *FILES], cwd=ROOT, check=True)
    try:
        (ROOT / "website/site.json").write_text(json.dumps(config, indent=4) + "\n", encoding="utf-8")
        (ROOT / "README.md").write_text(readme, encoding="utf-8")
        for script, options in (("build-website.py", ["--skip-wasm"]), ("prepare-fly.py", [])):
            subprocess.run([sys.executable, str(ROOT / "scripts" / script), *options], cwd=ROOT, check=True)
    except Exception:
        for name, content in original.items():
            (ROOT / name).write_text(content, encoding="utf-8")
        raise
    if subprocess.run(["git", "diff", "--quiet", "--", *FILES], cwd=ROOT).returncode:
        subprocess.run(["git", "-c", "user.name=Codex", "-c", "user.email=codex@localhost",
                        "commit", "--only", "-m", "Publish official token details", "--", *FILES], cwd=ROOT, check=True)
    logs = ROOT / "target/token-publication"
    logs.mkdir(parents=True, exist_ok=True)

    def publish(name, command, directory):
        with (logs / f"{name}.log").open("w", encoding="utf-8") as stream:
            return subprocess.run(command, cwd=directory, stdout=stream, stderr=subprocess.STDOUT,
                                  env={**os.environ, "GIT_TERMINAL_PROMPT": "0"}).returncode

    with ThreadPoolExecutor(max_workers=2) as pool:
        jobs = [("github", pool.submit(publish, "github", ["git", "push"], ROOT)),
                ("fly", pool.submit(publish, "fly", [str(fly), "deploy", "--remote-only", "--ha=false", "--yes"], ROOT / "target/fly-website"))]
        failed = [name for name, job in jobs if job.result() != 0]
    if failed:
        raise RuntimeError(f"Publication failed for {', '.join(failed)}. Inspect {logs}; source details are retained for retry.")
    with urllib.request.urlopen("https://diggerchain.xyz/site.json", timeout=20) as response:
        live = json.load(response)
    if live["token"] != config["token"] or live["links"] != config["links"]:
        raise RuntimeError("The live configuration differs from the supplied details; publication is not confirmed.")
    print(f"Token details published to the website and GitHub in {time.monotonic() - started:.1f}s.")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"Token publication failed: {error}") from error
