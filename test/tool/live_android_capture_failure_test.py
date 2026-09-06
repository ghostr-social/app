import tempfile
import unittest
from live_android_capture_fixture import CaptureFixture


class LiveAndroidCaptureFailureTest(unittest.TestCase):
    def test_recording_failure_cleans_owned_files_and_preserves_the_failure(self):
        with tempfile.TemporaryDirectory(prefix="ghostr-capture-failure-") as name:
            fixture = CaptureFixture(name)
            result = fixture.run("record-failure")
            self.assertNotEqual(result.returncode, 0)
            commands = fixture.commands()
            recorded = next(command[-1] for command in commands if "screenrecord" in command)
            pushed = next(command[-1] for command in commands if command[2] == "push")
            removed = {command[-1] for command in commands if command[2:4] == ["shell", "rm"]}
            self.assertEqual(removed, {recorded, pushed})
            self.assertFalse((fixture.output / "display-00.mp4").exists())


if __name__ == "__main__":
    unittest.main()
