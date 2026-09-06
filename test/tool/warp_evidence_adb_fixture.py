import os
from pathlib import Path
import signal
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def run_disconnected_capture(folder):
    folder = Path(folder)
    adb = folder / "adb"
    adb.write_text(FAKE_ADB)
    adb.chmod(0o755)
    environment = {**os.environ, "PATH": f"{folder}:{os.environ['PATH']}",
                   "WARP_EVIDENCE_ROOT": str(folder / "evidence"),
                   "WARP_LOGCAT_TIMEOUT_SECONDS": "0.05"}
    command = ["sh", str(ROOT / "tool/run_warp_evidence.sh"), "fixture-phone",
               "disconnect", "sh", "-c", "echo WARP_QOE fixture; exit 7"]
    process = subprocess.Popen(command, env=environment, text=True,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               start_new_session=True)
    try:
        output, error = process.communicate(timeout=5)
        return process.returncode, output, error
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGTERM)
        output, error = process.communicate(timeout=5)
        return 124, output, error


FAKE_ADB = '''#!/usr/bin/env python3
import sys, time
if "logcat" in sys.argv:
    time.sleep(30)
elif "devices" in sys.argv:
    print("fixture-phone\\tdevice")
else:
    print("0")
'''
