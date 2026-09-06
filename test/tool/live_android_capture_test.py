import json
import tempfile
import unittest
from live_android_capture_fixture import CaptureFixture


class LiveAndroidCaptureTest(unittest.TestCase):
    def test_capture_uses_explicit_device_and_removes_only_owned_remote_files(self):
        with tempfile.TemporaryDirectory(prefix="ghostr-capture-test-") as name:
            fixture = CaptureFixture(name)
            result = fixture.run()
            self.assertEqual(result.returncode, 0, result.stderr)
            commands = fixture.commands()
            self.assertTrue(all(command[:2] == ["-s", "fixture-phone"] for command in commands))
            recorded = next(command[-1] for command in commands if "screenrecord" in command)
            pushed = next(command[-1] for command in commands if command[2] == "push")
            removed = {command[-1] for command in commands if command[2:4] == ["shell", "rm"]}
            self.assertEqual(removed, {recorded, pushed})
            self.assertTrue((fixture.output / "display-00.mp4").exists())
            metadata = json.loads((fixture.output / "segments.json").read_text())[0]
            self.assertIn("encoder", metadata["limitation"])
            self.assertEqual(metadata["status"], "complete")


if __name__ == "__main__":
    unittest.main()
