from pathlib import Path
import tempfile
import unittest

from warp_evidence_adb_fixture import run_disconnected_capture


class DisconnectedEvidenceTest(unittest.TestCase):
    def test_optional_logcat_cannot_hide_the_journey_exit_or_hang_cleanup(self):
        with tempfile.TemporaryDirectory() as folder:
            status, output, error = run_disconnected_capture(folder)
            self.assertEqual(status, 7, f"optional ADB blocked the journey: {error}")
            self.assertIn("WARP_EVIDENCE_DIR=", output)
            evidence = next((Path(folder) / "evidence").iterdir())
            self.assertEqual((evidence / "exit.txt").read_text().strip(), "7")
            self.assertIn("source_changed=false", (evidence / "summary.txt").read_text())
            self.assertIn("WARP_QOE fixture", (evidence / "markers.log").read_text())


if __name__ == "__main__":
    unittest.main()
