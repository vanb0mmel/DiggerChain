import copy
import json
import tempfile
import unittest
from pathlib import Path

from testnet_check import BASE_FIELD, CHAIN_ID, DEFAULT_ACCOUNT, SCALAR_FIELD, SELECTOR, SIGNATURE, calldata, load_package


class TestnetPackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "out/Verifier.sol").mkdir(parents=True)
        self.inputs = ["2", "0", str(CHAIN_ID), "0", "1", "23", "12", "0", "2", "2", "512"]
        self.evm = {"function": SIGNATURE, "proof": ["1"] * 8, "public_inputs": self.inputs}
        self.report = {"experiment": "test-fixture", "native_verification": "passed", "cryptographic_proof": True,
                       "production_ready": False, "public_inputs_decimal": self.inputs}
        self.artifact = {"methodIdentifiers": {SIGNATURE: SELECTOR}, "bytecode": {"object": "0x6000"},
                         "deployedBytecode": {"object": "0x6001"}}
        self.write()

    def write(self):
        for name, data in (("report.json", self.report), ("evm.json", self.evm),
                           ("out/Verifier.sol/Verifier.json", self.artifact)):
            (self.root / name).write_text(json.dumps(data), encoding="utf-8")
        (self.root / "proof.bin").write_bytes(b"test proof")

    def test_static_abi_encoding_and_altered_score_are_bound(self):
        package = load_package(self.root, DEFAULT_ACCOUNT)
        raw = bytes.fromhex(package["validData"][2:])
        changed = bytes.fromhex(package["alteredData"][2:])
        self.assertEqual(len(raw), 612)
        self.assertEqual(raw[:4].hex(), SELECTOR)
        self.assertEqual(int.from_bytes(raw[4 + 13 * 32:4 + 14 * 32], "big"), 23)
        self.assertEqual(int.from_bytes(changed[4 + 13 * 32:4 + 14 * 32], "big"), 24)
        self.assertEqual(package["chainId"], CHAIN_ID)

    def test_wrong_network_is_rejected(self):
        self.inputs[2] = "4663"
        self.write()
        with self.assertRaisesRegex(ValueError, "testnet"):
            load_package(self.root, DEFAULT_ACCOUNT)

    def test_mismatched_proof_report_is_rejected(self):
        self.evm = copy.deepcopy(self.evm)
        self.evm["public_inputs"][5] = "999"
        self.write()
        with self.assertRaisesRegex(ValueError, "disagree"):
            load_package(self.root, DEFAULT_ACCOUNT)

    def test_noncanonical_or_malformed_words_are_rejected(self):
        for bad in ("-1", "0x1", "1e3", "9" * 79, str(BASE_FIELD)):
            with self.assertRaises(ValueError):
                calldata([bad] + ["1"] * 7, self.inputs)
        with self.assertRaises(ValueError):
            calldata(["1"] * 8, [str(SCALAR_FIELD)] + self.inputs[1:])

    def test_wrong_abi_or_unlinked_bytecode_is_rejected(self):
        self.artifact["bytecode"]["object"] = "0x__unlinked__"
        self.write()
        with self.assertRaises(ValueError):
            load_package(self.root, DEFAULT_ACCOUNT)
        self.artifact["bytecode"]["object"] = "0x6000"
        self.artifact["methodIdentifiers"][SIGNATURE] = "deadbeef"
        self.write()
        with self.assertRaisesRegex(ValueError, "ABI"):
            load_package(self.root, DEFAULT_ACCOUNT)

    def test_unverified_packages_are_rejected(self):
        self.report["native_verification"] = "not_run"
        self.write()
        with self.assertRaisesRegex(ValueError, "native"):
            load_package(self.root, DEFAULT_ACCOUNT)


if __name__ == "__main__":
    unittest.main()
