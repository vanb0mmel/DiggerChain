#!/usr/bin/env python3
"""Build the public static website and its shared-Rust practice demo."""

import argparse
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib
import zipfile

from website_content import render_site


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "website"
OUTPUT = ROOT / "target" / "website"
DEFAULT_ARCHIVE = ROOT / "target/dist/diggerchain-0.1.0-alpha.1-linux-x86_64.zip"
PUBLIC_ARCHIVE = "diggerchain-linux-alpha.zip"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-archive", type=Path, default=DEFAULT_ARCHIVE)
    parser.add_argument("--skip-wasm", action="store_true", help="Reuse an already built WASM module while editing the site")
    args = parser.parse_args()
    archive = args.native_archive.resolve()
    if not archive.is_file() or not zipfile.is_zipfile(archive):
        parser.error("Provide the Linux alpha ZIP with --native-archive PATH.")
    if not args.skip_wasm:
        cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
        bindgen = shutil.which("wasm-bindgen")
        if not bindgen and (cargo_home / "bin/wasm-bindgen").is_file():
            bindgen = str(cargo_home / "bin/wasm-bindgen")
        if not bindgen:
            parser.error("Install the matching tool: cargo install wasm-bindgen-cli --version 0.2.129 --locked -j 2")
        expected = tomllib.loads((ROOT / "crates/diggerchain-web/Cargo.toml").read_text())["dependencies"]["wasm-bindgen"].lstrip("=")
        actual = subprocess.check_output([bindgen, "--version"], text=True).strip().split()[-1]
        if actual != expected:
            parser.error(f"wasm-bindgen {expected} is required; found {actual}.")
        subprocess.run([
            "cargo", "build", "--profile", "web", "--locked", "-j", "2",
            "-p", "diggerchain-web", "--target", "wasm32-unknown-unknown",
        ], cwd=ROOT, check=True)
    elif not all((OUTPUT / "pkg" / name).is_file() for name in ("diggerchain_web.js", "diggerchain_web_bg.wasm")):
        parser.error("Build once without --skip-wasm first.")

    # Only allowlisted public files are copied; saves, setup material, and the repo stay private.
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="website-", dir=OUTPUT.parent) as temporary:
        staging = Path(temporary) / "public"
        (staging / "pkg").mkdir(parents=True)
        if args.skip_wasm:
            for name in ("diggerchain_web.js", "diggerchain_web_bg.wasm"):
                shutil.copyfile(OUTPUT / "pkg" / name, staging / "pkg" / name)
        else:
            subprocess.run([
                bindgen, "--target", "web", "--no-typescript",
                "--out-dir", str(staging / "pkg"),
                str(ROOT / "target/wasm32-unknown-unknown/web/diggerchain_web.wasm"),
            ], cwd=ROOT, check=True)
        for name in ("site.css", "app.js", "theme.js", "demo.js"):
            shutil.copyfile(SOURCE / name, staging / name)
        render_site(staging)
        shutil.copytree(SOURCE / "assets", staging / "assets")
        downloads = staging / "downloads"
        downloads.mkdir()
        shutil.copyfile(archive, downloads / PUBLIC_ARCHIVE)
        with archive.open("rb") as source:
            digest = hashlib.file_digest(source, "sha256").hexdigest()
        (downloads / f"{PUBLIC_ARCHIVE}.sha256").write_text(f"{digest}  {PUBLIC_ARCHIVE}\n")
        if OUTPUT.exists():
            shutil.rmtree(OUTPUT)
        staging.rename(OUTPUT)

    bundle = ROOT / "target/dist/diggerchain-website-preview.zip"
    bundle.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(bundle, "w", zipfile.ZIP_DEFLATED) as zipped:
        for path in sorted(OUTPUT.rglob("*")):
            if path.is_file():
                zipped.write(path, path.relative_to(OUTPUT))
    core_bytes = sum(path.stat().st_size for path in OUTPUT.rglob("*") if path.is_file() and "downloads" not in path.parts)
    print(f"Static website: {OUTPUT}")
    print(f"Website and demo assets: {core_bytes:,} bytes; native download is separate.")
    print(f"Host-ready ZIP: {bundle}")
    print("Preview: python3 scripts/serve-website.py")


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        sys.exit(error.returncode)
