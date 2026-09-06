"""Unavailable pins remain failures in the complete requested population."""
import json
import unittest

from live_video_summary_fixture import run_summary, sample


class MissingPinnedVideosTest(unittest.TestCase):
    def test_missing_pins_are_not_removed_from_percentiles(self):
        report = {
            "records": [{"type": "direct_player", "firstFrameMs": 1}],
            "samples": [sample("a", 50), sample("b", 150), sample("c", 200)],
            "failures": ["Relay unavailable for d", "Cannot map e", "Missing f"],
            "droppedRecords": 0,
        }
        result = run_summary(report, "--mode", "pinned", "--pins", "a,b,c,d,e,f")
        self.assertEqual(result.returncode, 0, result.stderr)
        summary = json.loads(result.stdout)
        pins = summary["pinned"]
        self.assertEqual(pins["requested"], 6)
        self.assertEqual(pins["missing_first_frames"], 3)
        self.assertEqual(pins["over_100ms_or_missing"], 5)
        self.assertEqual(pins["observed_callback_ms"],
                         {"median": 150, "p95": 200, "worst": 200})
        self.assertEqual(pins["all_requested_callback_ms"],
                         {"median": None, "p95": None, "worst": None})
        self.assertFalse(summary["measurement"]["physical_presentation_verified"])
        self.assertEqual(summary["failures"], report["failures"])


if __name__ == "__main__":
    unittest.main()
