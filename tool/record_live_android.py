#!/usr/bin/env python3
"""Capture a live Android journey with explicit device and clock alignment."""
import argparse
import datetime
import json
from pathlib import Path
import re
import subprocess
import time
import uuid

LIMITATION = ("Virtual display capture adds encoder work and does not independently "
              "prove physical panel presentation.")


class Capture:
    def __init__(self, args):
        if not args.clock_binary.is_file():
            raise ValueError("clock binary must be a regular file")
        self.args = args
        self.adb = [args.adb, "-s", args.serial]
        self.remote = f"/data/local/tmp/ghostr-live-{uuid.uuid4().hex}"
        self.owned = set()
        self.segments = []

    def command(self, *parts, timeout=30):
        return subprocess.run(self.adb + list(parts), text=True, capture_output=True,
                              check=True, timeout=timeout).stdout

    def source(self):
        if not self.args.runner_log.exists():
            return ""
        return self.args.runner_log.read_text(errors="replace")

    def wait_for_bootstrap(self):
        deadline = time.monotonic() + self.args.bootstrap_timeout
        while time.monotonic() < deadline:
            found = re.search(r'I/flutter\s*\(\s*(\d+)\): WARP_LIVE \{"type":"bootstrap_started"', self.source())
            if found:
                return self.validate_pid(found.group(1))
            time.sleep(1)
        raise RuntimeError("No bootstrap; did not start recording")

    def validate_pid(self, expected):
        actual = self.command("shell", "pidof", "app.ghostr").strip()
        if actual != expected:
            raise RuntimeError("Bootstrap app no longer active; did not start recording")
        return expected

    def sample_clock(self, name):
        clock = self.command("shell", self.remote + "-clock", timeout=10)
        (self.args.output / name).write_text(clock)

    def save_segments(self):
        (self.args.output / "segments.json").write_text(json.dumps(self.segments, indent=2) + "\n")

    def record_segment(self, index, pid):
        remote = f"{self.remote}-{index:02d}.mp4"
        self.owned.add(remote)
        self.sample_clock(f"clock-{index:02d}.jsonl")
        elapsed = re.findall(r'"elapsedMs":(\d+)', self.source())
        metadata = {"index": index, "pid": pid, "status": "started", "limitation": LIMITATION,
                    "hostStartedUtc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                    "latestAppElapsedMs": int(elapsed[-1]) if elapsed else None}
        self.segments.append(metadata)
        self.save_segments()
        print(f"Recording segment {index}, app elapsed {metadata['latestAppElapsedMs']}", flush=True)
        self.record_file(remote, index)
        metadata["status"] = "complete"
        self.save_segments()

    def record_file(self, remote, index):
        with (self.args.output / f"screenrecord-{index:02d}.log").open("w") as log:
            subprocess.run(self.adb + ["shell", "screenrecord", "--size", "720x1600",
                           "--bit-rate", "4000000", "--time-limit", "180", remote],
                           stdout=log, stderr=subprocess.STDOUT, check=True, timeout=195)
        self.command("pull", remote, str(self.args.output / f"display-{index:02d}.mp4"), timeout=60)
        self.remove(remote)

    def remove(self, remote):
        self.command("shell", "rm", remote, timeout=10)
        self.owned.remove(remote)

    def record(self, pid):
        for index in range(self.args.max_segments):
            if '"type":"result"' in self.source():
                break
            self.validate_pid(pid)
            self.record_segment(index, pid)
        self.sample_clock("clock-after.jsonl")

    def run(self):
        pid = self.wait_for_bootstrap()
        self.args.output.mkdir(parents=True, exist_ok=True)
        remote_clock = self.remote + "-clock"
        self.owned.add(remote_clock)
        try:
            self.command("push", str(self.args.clock_binary), remote_clock)
            self.with_logcat(pid)
        finally:
            for remote in list(self.owned):
                self.remove(remote)

    def with_logcat(self, pid):
        with (self.args.output / "flutter-epoch.log").open("w") as log:
            process = subprocess.Popen(self.adb + ["logcat", "-v", "epoch", "-s", "flutter:I"], stdout=log)
            try:
                self.record(pid)
            finally:
                process.terminate()
                process.wait(timeout=10)


def arguments():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--serial", required=True)
    parser.add_argument("--runner-log", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--clock-binary", type=Path, required=True)
    parser.add_argument("--adb", default="adb")
    parser.add_argument("--max-segments", type=int, default=6, choices=range(1, 13))
    parser.add_argument("--bootstrap-timeout", type=float, default=900)
    return parser.parse_args()


if __name__ == "__main__":
    Capture(arguments()).run()
