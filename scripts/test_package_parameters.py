import hashlib
import json
import tempfile
import unittest
import zipfile
from pathlib import Path

from package_parameters import package


class ParametersPackageTest(unittest.TestCase):
    def test_packaging_requires_a_matching_player_page_and_excludes_private_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            parameters = root / "parameters"
            (parameters / "wallet").mkdir(parents=True)
            manifest = {"scheme": "groth16-bn254-reward-v1"}
            (parameters / "src").mkdir()
            (parameters / "src/Verifier.sol").write_bytes(b"public source")
            manifest["verifier_source_sha256"] = hashlib.sha256(b"public source").hexdigest()
            for name, field in [("proving-key.bin", "proving_key_sha256"), ("verifying-key.bin", "verifying_key_sha256")]:
                (parameters / name).write_bytes(name.encode())
                manifest[field] = hashlib.sha256(name.encode()).hexdigest()
            (parameters / "manifest.json").write_text(json.dumps(manifest))
            page = {"verifyingKeySha256": manifest["verifying_key_sha256"], "playerMode": True, "expectedAccount": None, "parametersPath": "./parameters", "chainId": 46630, "verifierAddress": "0x" + "01"*20, "rewardsAddress": "0x" + "02"*20}
            (parameters / "wallet/package.json").write_text(json.dumps(page))
            template = (Path(__file__).resolve().parents[1] / "contracts/wallet.html").read_text(encoding="utf-8")
            (parameters / "wallet/index.html").write_bytes(template.replace("{{PAYLOAD}}", json.dumps(page).replace("<", "\\u003c")).encode())
            (parameters / "private-save.json").write_text("must not be packaged")
            archive_path = package(parameters, "public", root / "dist")
            with zipfile.ZipFile(archive_path) as archive:
                self.assertNotIn("private-save.json", archive.namelist())
                self.assertIn("wallet.html", archive.namelist())
                self.assertEqual(archive.read("parameters/proving-key.bin"), b"proving-key.bin")
            page["playerMode"] = False
            (parameters / "wallet/package.json").write_text(json.dumps(page))
            with self.assertRaises(ValueError):
                package(parameters, "operator", root / "dist")
            page["playerMode"] = True
            (parameters / "wallet/package.json").write_text(json.dumps(page))
            (parameters / "proving-key.bin").write_bytes(b"wrong")
            with self.assertRaises(ValueError):
                package(parameters, "wrong-key", root / "dist")


if __name__ == "__main__":
    unittest.main()
