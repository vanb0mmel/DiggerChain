import copy
import unittest

from publish_token import END, START, update_readme
from website_content import validate_config


class TokenPublicationTest(unittest.TestCase):
    def test_details_replace_only_the_readme_block(self):
        config = {
            "token": {"symbol": "DCG", "chainId": 4663, "contractAddress": "0x" + "12" * 20},
            "links": {"x": "https://x.com/example", "ponsFamily": "https://ponsfamily.com/example"},
        }
        validate_config(config)
        original = "Before\n" + START + "Old content" + END + "\nAfter"
        result = update_readme(original, config)
        self.assertTrue(result.startswith("Before\n" + START))
        self.assertTrue(result.endswith(END + "\nAfter"))
        self.assertIn(config["token"]["contractAddress"], result)
        self.assertIn(config["links"]["ponsFamily"], result)
        self.assertEqual(update_readme(result, config), result)
        for address in ("not-an-address", "0x123"):
            invalid = copy.deepcopy(config)
            invalid["token"]["contractAddress"] = address
            with self.assertRaises(ValueError):
                validate_config(invalid)
        invalid = copy.deepcopy(config)
        invalid["links"]["x"] = "https://x.com/example\nextra"
        with self.assertRaises(ValueError):
            validate_config(invalid)
        with self.assertRaises(ValueError):
            update_readme("Missing token block", config)


if __name__ == "__main__":
    unittest.main()
