#!/usr/bin/env python3
"""Summarize a physical-phone report without dropping unavailable videos."""
import argparse
import json
from pathlib import Path

from live_video_summary import summarize


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--mode", choices=("fresh", "pinned"), required=True)
    parser.add_argument("--pins", default="", help="Complete comma-separated requested IDs")
    parser.add_argument("--fresh-count", type=int, default=20,
                        help="Requested count when a fresh run ends before its corpus record")
    args = parser.parse_args()
    try:
        report = json.loads(args.report.read_text())
        pins = [value.strip() for value in args.pins.split(",") if value.strip()]
        result = summarize(report, args.mode, pins, args.fresh_count)
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.error(str(error))
    print(json.dumps(result, indent=2, allow_nan=False))


if __name__ == "__main__":
    main()
