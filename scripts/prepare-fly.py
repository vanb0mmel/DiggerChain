#!/usr/bin/env python3
"""Stage a Fly build context from the already allowlisted public website only."""

from pathlib import Path
import shutil
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "target/website"
DESTINATION = ROOT / "target/fly-website"


def main():
    for name in ("index.html", "pkg/diggerchain_web_bg.wasm", "llms.txt"):
        if not (SOURCE / name).is_file():
            raise SystemExit("Build the website before preparing its Fly context.")
    with tempfile.TemporaryDirectory(prefix="fly-stage-", dir=ROOT / "target") as temporary:
        staging = Path(temporary) / "context"
        staging.mkdir()
        shutil.copytree(SOURCE, staging / "public")
        for name in ("Dockerfile", "nginx.conf", "fly.toml"):
            shutil.copyfile(ROOT / "deploy/fly" / name, staging / name)
        (staging / ".dockerignore").write_text("*\n!Dockerfile\n!nginx.conf\n!public/\n!public/**\n")
        if DESTINATION.exists():
            shutil.rmtree(DESTINATION)
        staging.rename(DESTINATION)
    print(f"Fly context prepared: {DESTINATION}")
    print("No remote action was performed. Review deploy/fly/README.md before deployment.")


if __name__ == "__main__":
    main()
