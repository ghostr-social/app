import tempfile
import unittest
from live_android_capture_fixture import CaptureFixture


class LiveAndroidCaptureClockInputTest(unittest.TestCase):
    def test_directory_cannot_be_pushed_as_a_clock_binary(self):
        with tempfile.TemporaryDirectory(prefix="ghostr-capture-clock-input-") as name:
            fixture = CaptureFixture(name)
            clock = fixture.folder / "clock"
            clock.unlink()
            clock.mkdir()
            result = fixture.run()
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("clock binary must be a regular file", result.stderr)
            self.assertFalse((fixture.folder / "commands").exists())


if __name__ == "__main__":
    unittest.main()
