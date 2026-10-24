#!/usr/bin/env python3
"""Package explicit native executables; never discover or include private saves."""

import argparse
import hashlib
import json
import re
import stat
import subprocess
import zipfile
from pathlib import Path


def dependency_notices(repo, include_proof=True, target=None):
    version = subprocess.check_output(["rustc", "-vV"], text=True, encoding="utf-8")
    host = next(line.split(": ", 1)[1] for line in version.splitlines() if line.startswith("host: "))
    target = target or host
    files, packages = {}, {}
    manifests = [repo / "Cargo.toml"]
    if include_proof:
        manifests.append(repo / "experiments/proof-lab/Cargo.toml")
    for manifest in manifests:
        metadata = json.loads(subprocess.check_output([
            "cargo", "metadata", "--format-version", "1", "--locked", "--offline",
            "--filter-platform", target, "--manifest-path", str(manifest)
        ], text=True, encoding="utf-8"))
        for dependency in metadata["packages"]:
            if not dependency["source"]:
                continue
            name = f"{dependency['name']}-{dependency['version']}"
            packages[name] = {key: dependency.get(key) for key in ("license", "authors", "repository")}
            directory = Path(dependency["manifest_path"]).parent
            for path in directory.iterdir():
                if path.is_file() and path.name.upper().startswith(("LICENSE", "COPYING", "COPYRIGHT", "NOTICE")):
                    files[f"THIRD_PARTY_LICENSES/{name}/{path.name}"] = path.read_bytes()
    sysroot = Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True, encoding="utf-8").strip())
    for name in ("COPYRIGHT-library.html", "COPYRIGHT.html"):
        path = sysroot / "share/doc/rust" / name
        if path.is_file():
            files[f"THIRD_PARTY_LICENSES/rust-toolchain/{name}"] = path.read_bytes()
    files["THIRD_PARTY_LICENSES/INDEX.json"] = (json.dumps({"rustc": version, "target": target, "packages": packages}, indent=2) + "\n").encode()
    return files


def package(game, proof, label, output, instructions, notices=None):
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,99}", label):
        raise ValueError("Invalid archive label.")
    executables = [game] if proof is None else [game, proof]
    if not all(executable.is_file() for executable in executables):
        raise ValueError("Every explicitly requested native executable must exist.")
    for executable in executables:
        with executable.open("rb") as stream:
            magic = stream.read(4)
        if not (magic.startswith((b"\x7fELF", b"MZ")) or magic in (b"\xfe\xed\xfa\xce", b"\xfe\xed\xfa\xcf", b"\xce\xfa\xed\xfe", b"\xcf\xfa\xed\xfe", b"\xca\xfe\xba\xbe", b"\xca\xfe\xba\xbf")):
            raise ValueError(f"Not a native executable: {executable.name}")
    output.mkdir(parents=True, exist_ok=True)
    destination = output / f"{label}.zip"
    files = {executable.name: executable.read_bytes() for executable in executables}
    files["INSTALL.md"] = instructions.read_bytes()
    if len(files) != len(executables) + 1:
        raise ValueError("Executable names must be distinct from each other and INSTALL.md.")
    files.update(notices or {})
    sums = "".join(f"{hashlib.sha256(data).hexdigest()}  {name}\n" for name, data in files.items())
    with zipfile.ZipFile(destination, "x", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in {**files, "SHA256SUMS": sums.encode()}.items():
            info = zipfile.ZipInfo(f"{label}/{name}")
            info.create_system = 3
            info.external_attr = (stat.S_IFREG | (0o755 if name in {item.name for item in executables} else 0o644)) << 16
            archive.writestr(info, data, compress_type=zipfile.ZIP_DEFLATED)
    checksum = hashlib.sha256(destination.read_bytes()).hexdigest()
    checksum_file = destination.with_suffix(".zip.sha256")
    with checksum_file.open("x", encoding="utf-8") as stream:
        stream.write(f"{checksum}  {destination.name}\n")
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--game", required=True, type=Path)
    parser.add_argument("--proof", type=Path, help="Include the separate development proof executable")
    parser.add_argument("--label", required=True)
    parser.add_argument("--output", type=Path, default=Path("target/dist"))
    parser.add_argument("--instructions", type=Path, help="Override the bundled installation guide")
    parser.add_argument("--target", help="Rust target triple used to build the executable")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    instructions = args.instructions or repo / "docs" / ("INSTALL.md" if args.proof else "ALPHA_INSTALL.md")
    try:
        print(package(args.game, args.proof, args.label, args.output, instructions, dependency_notices(repo, include_proof=args.proof is not None, target=args.target)))
    except (OSError, ValueError, subprocess.CalledProcessError, zipfile.BadZipFile) as error:
        parser.exit(1, f"Cannot package native candidate: {error}\n")


if __name__ == "__main__":
    main()
