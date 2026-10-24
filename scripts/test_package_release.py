import hashlib
import json
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

from package_release import dependency_notices, package


class PackageTest(unittest.TestCase):
    def test_alpha_omits_proof_and_keeps_game_executable(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            game = root / "diggerchain"
            game.write_bytes(b"\x7fELFgame")
            instructions = root / "ALPHA_INSTALL.md"
            instructions.write_text("Alpha installation")
            (root / "diggerchain-proof").write_bytes(b"private development binary")
            output = package(game, None, "alpha", root / "dist", instructions)
            with zipfile.ZipFile(output) as archive:
                self.assertEqual(set(archive.namelist()), {"alpha/diggerchain", "alpha/INSTALL.md", "alpha/SHA256SUMS"})
                self.assertEqual((archive.getinfo("alpha/diggerchain").external_attr >> 16) & 0o777, 0o755)
                self.assertEqual(archive.read("alpha/INSTALL.md"), b"Alpha installation")

    def test_archive_contains_only_explicit_binaries_instructions_and_checksums(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            game, proof, instructions = (root / name for name in ("diggerchain", "diggerchain-proof", "INSTALL.md"))
            game.write_bytes(b"\x7fELFgame")
            proof.write_bytes(b"\x7fELFproof")
            instructions.write_text("Instructions")
            (root / "private-save.json").write_text("must not be packaged")
            registry = root / "registry"
            registry.mkdir()
            (registry / "LICENSE").write_bytes(b"upstream license")
            metadata = {"packages": [{"name": "dependency", "version": "1", "source": "registry", "manifest_path": str(registry / "Cargo.toml"), "license": "MIT"}]}
            def command(arguments, **_):
                if arguments[0] == "cargo":
                    return json.dumps(metadata)
                return "host: test-host\n" if "-vV" in arguments else str(root)
            with patch("package_release.subprocess.check_output", side_effect=command):
                notices = dependency_notices(root)
            output = package(game, proof, "candidate", root / "dist", instructions, notices)
            with zipfile.ZipFile(output) as archive:
                self.assertEqual(set(archive.namelist()), {"candidate/diggerchain", "candidate/diggerchain-proof", "candidate/INSTALL.md", "candidate/SHA256SUMS", "candidate/THIRD_PARTY_LICENSES/INDEX.json", "candidate/THIRD_PARTY_LICENSES/dependency-1/LICENSE"})
                self.assertEqual(archive.read("candidate/THIRD_PARTY_LICENSES/dependency-1/LICENSE"), b"upstream license")
                self.assertIn(hashlib.sha256(b"\x7fELFgame").hexdigest(), archive.read("candidate/SHA256SUMS").decode())
            self.assertTrue(output.with_suffix(".zip.sha256").read_text().startswith(hashlib.sha256(output.read_bytes()).hexdigest()))
            with self.assertRaises(FileExistsError):
                package(game, proof, "candidate", root / "dist", instructions)


if __name__ == "__main__":
    unittest.main()
