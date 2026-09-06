#!/usr/bin/env python3
"""Produce Flutter defines from real samples retained by previous phone runs."""
import json
from pathlib import Path
import sys


def collect(root):
    events, urls = set(), set()
    for path in root.rglob("markers.log"):
        with path.open(errors="replace") as stream:
            for line in stream:
                add_sample(line, events, urls)
    return {"eventIds": sorted(events), "urls": sorted(urls)}


def add_sample(line, events, urls):
    if "WARP_LIVE " not in line or '"sample"' not in line or len(line) > 65536:
        return
    try:
        sample = json.loads(line.split("WARP_LIVE ", 1)[1])
    except json.JSONDecodeError:
        return
    if not isinstance(sample, dict) or sample.get("type") != "sample":
        return
    for key, target in [("eventId", events), ("url", urls)]:
        value = sample.get(key)
        if isinstance(value, str) and value:
            target.add(value)


if __name__ == "__main__":
    print(json.dumps({"LIVE_VIDEO_PRIOR_CORPUS": json.dumps(collect(Path(sys.argv[1])))}))
