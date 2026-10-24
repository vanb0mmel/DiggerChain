import unittest

from website_content import download_markup


class DownloadContentTest(unittest.TestCase):
    def test_unavailable_platform_and_safe_downloads(self):
        release = {"downloads": [
            {"platform": "Linux", "url": "./downloads/game.zip", "checksum": "./downloads/game.zip.sha256"},
            {"platform": "Windows", "url": None, "checksum": None},
        ]}
        markup = download_markup(release)
        self.assertIn('href="./downloads/game.zip"', markup)
        self.assertIn("Windows</dt><dd>Coming Soon", markup)
        release["downloads"][0]["url"] = "javascript:alert(1)"
        with self.assertRaises(ValueError):
            download_markup(release)
        release["downloads"][0]["url"] = None
        with self.assertRaises(ValueError):
            download_markup(release)


if __name__ == "__main__":
    unittest.main()
