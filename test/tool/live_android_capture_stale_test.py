import tempfile
import unittest
from live_android_capture_fixture import CaptureFixture


class LiveAndroidCaptureStaleTest(unittest.TestCase):
    def test_stale_bootstrap_cannot_record_another_app_process(self):
        with tempfile.TemporaryDirectory(prefix="ghostr-capture-stale-") as name:
            fixture = CaptureFixture(name)
            result = fixture.run("stale")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Bootstrap app no longer active", result.stderr)
            self.assertEqual(fixture.commands(), [["-s", "fixture-phone", "shell", "pidof", "app.ghostr"]])


if __name__ == "__main__":
    unittest.main()
