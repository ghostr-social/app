"""Run the public summary command against a small journey report."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def sample(event, latency, scenario="pinned_warp"):
    return {
        "eventId": event, "scenario": scenario, "firstFrameMs": latency,
        "renderedAndMoving": latency is not None, "longestFreezeMs": 0,
        "focusChanged": False, "unavailableVisible": False,
    }


def run_summary(report, *arguments):
    with tempfile.TemporaryDirectory(prefix="ghostr-summary-") as directory:
        path = Path(directory) / "report.json"
        path.write_text(json.dumps(report))
        return subprocess.run(
            [sys.executable, str(ROOT / "tool/summarize_live_video.py"),
             "--report", str(path), *arguments],
            capture_output=True, text=True, timeout=5, check=False,
        )
