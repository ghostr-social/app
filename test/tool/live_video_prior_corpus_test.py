import json
from pathlib import Path
import subprocess
import tempfile
import unittest


class PriorCorpusTest(unittest.TestCase):
    def test_real_marker_files_produce_cross_run_event_and_media_exclusions(self):
        script = Path(__file__).resolve().parents[2] / "tool/live_video_prior_corpus.py"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = {"type": "sample", "eventId": "first", "url": "https://media.example/one.mp4"}
            repost = {**first, "eventId": "repost"}
            (root / "markers.log").write_text(
                "unrelated output\nWARP_LIVE {truncated\n"
                + "\n".join("WARP_LIVE " + json.dumps(row) for row in [first, repost, first])
            )
            result = subprocess.run(
                ["python3", str(script), str(root)], capture_output=True, text=True
            )
        self.assertEqual(result.returncode, 0, result.stderr)
        corpus = json.loads(json.loads(result.stdout)["LIVE_VIDEO_PRIOR_CORPUS"])
        self.assertEqual(corpus["eventIds"], ["first", "repost"])
        self.assertEqual(corpus["urls"], ["https://media.example/one.mp4"])


if __name__ == "__main__":
    unittest.main()
