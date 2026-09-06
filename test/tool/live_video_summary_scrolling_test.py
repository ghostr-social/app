"""Cold startup stays separate and unfinished scrolling remains in the totals."""
import json
import unittest

from live_video_summary_fixture import run_summary, sample


class ScrollingPopulationTest(unittest.TestCase):
    def test_unfinished_backtracking_and_rapid_settle_are_counted(self):
        report = {
            "records": [{"type": "fresh_corpus", "requested": 5, "sampled": 5,
                         "hosts": {"origin.test": 5}},
                        {"type": "configuration", "coldCache": True},
                        {"type": "isolated_cache", "key": "fresh-example"}],
            "samples": [sample("a", 1000, "startup"),
                        sample("b", 10, "browse"), sample("c", 80, "browse"),
                        sample("d", 100, "browse"), sample("e", None, "browse"),
                        sample("d", 40, "warm_return"),
                        sample("c", None, "warm_return")],
            "failures": ["Could not finish backward navigation"],
        }
        result = run_summary(report, "--mode", "fresh")
        self.assertEqual(result.returncode, 0, result.stderr)
        summary = json.loads(result.stdout)
        scroll = summary["scrolling"]
        self.assertEqual(scroll["requested"], 9)
        self.assertEqual(scroll["missing_first_frames"], 5)
        self.assertEqual(scroll["over_100ms_or_missing"], 5)
        self.assertEqual(scroll["observed_callback_ms"],
                         {"median": 60, "p95": 100, "worst": 100})
        self.assertIsNone(scroll["all_requested_callback_ms"]["median"])
        self.assertEqual(summary["cold_startup"]["observed_callback_ms"]["worst"], 1000)
        self.assertEqual(summary["backward"]["requested"], 4)
        self.assertEqual(summary["rapid_settle"]["missing_first_frames"], 1)
        self.assertEqual(summary["cache"], {"cold": True, "key": "fresh-example"})


if __name__ == "__main__":
    unittest.main()
