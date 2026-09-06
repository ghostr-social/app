"""Attach to the explicitly installed app without Flutter's reinstall fallback."""

import re
import subprocess
import sys
import time
from urllib.parse import urlsplit, urlunsplit


def adb(*arguments):
    return subprocess.check_output(
        [sys.argv[1], "-s", sys.argv[2], *arguments], text=True, timeout=10
    ).strip()


def service():
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        pid = app_pid()
        if not pid:
            time.sleep(0.2)
            continue
        output = adb("logcat", "-d", "--pid=" + pid, "-s", "flutter:I")
        matches = re.findall(r"The Dart VM service is listening on (http://\S+)", output)
        if matches:
            return urlsplit(matches[-1])
        time.sleep(0.2)
    raise RuntimeError("Installed profile app did not expose a VM service within 30 seconds")


def app_pid():
    try:
        return adb("shell", "pidof", "app.ghostr")
    except subprocess.CalledProcessError as error:
        if error.returncode != 1:
            raise
        return ""


if __name__ == "__main__":
    uri = service()
    forwarded = adb("forward", "tcp:0", "tcp:" + str(uri.port))
    print(urlunsplit((uri.scheme, "127.0.0.1:" + forwarded, uri.path, uri.query, uri.fragment)))
