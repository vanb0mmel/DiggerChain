"""Focused public-artifact and immutable-reference boundary checks."""

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import reward_wallet as wallet


class RewardWalletTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.parameters = self.root / "parameters"
        self.parameters.mkdir()
        self.manifest = {
            "scheme": "groth16-bn254-reward-v1", "public_input_names": wallet.NAMES,
            "max_actions": 512, "setup_assumption": "single_operator_trusted_setup",
            "verifying_key_sha256": wallet.fingerprint(b"key"), "verifier_source_sha256": wallet.fingerprint(b"source"),
        }
        (self.parameters / "verifying-key.bin").write_bytes(b"key")
        (self.parameters / "src").mkdir()
        (self.parameters / "src/Verifier.sol").write_bytes(b"source")
        self.verifier = {"methodIdentifiers": {"verify(uint256[8],uint256[14])": "b11c6afb"},
                         "bytecode": {"object": "0x6000"}, "deployedBytecode": {"object": "0x6000"}}
        self.rewards = {"methodIdentifiers": {"register(uint256[8],uint256[14])": "be706fea"},
                        "bytecode": {"object": "0x6000"},
                        "deployedBytecode": {"object": "0x" + "00" * 64, "immutableReferences": {"1": [{"start": 0, "length": 32}]}}}
        self.write_artifacts()
        self.patcher = patch.object(wallet, "REPO", self.root)
        self.patcher.start()

    def tearDown(self):
        self.patcher.stop()
        self.temporary.cleanup()

    def write_artifacts(self):
        files = [(self.parameters / "manifest.json", self.manifest),
                 (self.parameters / "out/Verifier.sol/Verifier.json", self.verifier),
                 (self.root / "contracts/out/DiggerChainRewards.sol/DiggerChainRewards.json", self.rewards)]
        for path, value in files:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps(value))

    def load(self):
        return wallet.load_package(self.parameters, wallet.ACCOUNT, {})

    def test_prepares_public_package_and_rejects_changed_key(self):
        package = self.load()
        self.assertEqual(package["chainId"], 46630)
        self.assertNotIn("secret", package)
        participants = wallet.load_package(self.parameters, None, {})
        self.assertIsNone(participants["expectedAccount"])
        self.assertEqual(participants["network"]["name"], "Robinhood Chain Testnet")
        (self.parameters / "verifying-key.bin").write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "disagree"):
            self.load()

    def test_rejects_an_experiment_or_wrong_reward_abi(self):
        self.manifest["scheme"] = "lab-v0"
        self.write_artifacts()
        with self.assertRaises(ValueError):
            self.load()
        self.manifest["scheme"] = "groth16-bn254-reward-v1"
        self.verifier["methodIdentifiers"] = {"verify(uint256[8],uint256[11])": "417ed909"}
        self.write_artifacts()
        with self.assertRaises(ValueError):
            self.load()

    def test_rejects_out_of_bounds_immutable_ranges_and_invalid_account(self):
        self.rewards["deployedBytecode"]["immutableReferences"]["1"][0]["start"] = 40
        self.write_artifacts()
        with self.assertRaisesRegex(ValueError, "immutable"):
            self.load()
        with self.assertRaisesRegex(ValueError, "wallet"):
            wallet.load_package(self.parameters, "0x0", {})


if __name__ == "__main__":
    unittest.main()
