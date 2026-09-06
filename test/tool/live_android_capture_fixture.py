import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]


class CaptureFixture:
    def __init__(self, folder):
        self.folder = Path(folder)
        self.output = self.folder / "capture"
        (self.folder / "runner.log").write_text(
            'I/flutter (42): WARP_LIVE {"type":"bootstrap_started","elapsedMs":1}\n')
        (self.folder / "clock").write_text("fixture clock")
        adb = self.folder / "adb"
        adb.write_text(FAKE_ADB)
        adb.chmod(0o755)

    def run(self, mode="success"):
        return subprocess.run([
            sys.executable, str(ROOT / "tool/record_live_android.py"),
            "--serial", "fixture-phone", "--runner-log", str(self.folder / "runner.log"),
            "--output", str(self.output), "--clock-binary", str(self.folder / "clock"),
            "--adb", str(self.folder / "adb"),
        ], env={**os.environ, "CAPTURE_TEST_DIR": str(self.folder), "CAPTURE_TEST_MODE": mode},
            text=True, capture_output=True, timeout=30)

    def commands(self):
        return [json.loads(line) for line in (self.folder / "commands").read_text().splitlines()]


FAKE_ADB = '''#!/usr/bin/env python3
import json, os, pathlib, sys, time
root = pathlib.Path(os.environ["CAPTURE_TEST_DIR"])
args = sys.argv[1:]
mode = os.environ["CAPTURE_TEST_MODE"]
with (root / "commands").open("a") as out:
    out.write(json.dumps(args) + "\\n")
cmd = args[2:]
if cmd[:2] == ["shell", "pidof"]:
    print("99" if mode == "stale" else "42")
elif cmd and cmd[0] == "logcat":
    time.sleep(25)
elif cmd[:2] == ["shell", "screenrecord"]:
    if mode == "record-failure": sys.exit(9)
    with (root / "runner.log").open("a") as out:
        out.write('I/flutter (42): WARP_LIVE {"type":"result","elapsedMs":2}\\n')
elif cmd and cmd[0] == "pull":
    pathlib.Path(cmd[-1]).write_bytes(b"fixture video")
elif cmd and cmd[0] == "shell" and len(cmd) == 2:
    print('{"realBeforeNs":100,"bootNs":10,"monotonicNs":10,"realAfterNs":102}')
'''
