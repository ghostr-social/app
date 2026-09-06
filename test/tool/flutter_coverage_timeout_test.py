"""Coverage runs retain their checks while allowing a slow storage deadline."""

import json
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


class FlutterCoverageTimeoutTest(unittest.TestCase):
    def test_default_and_explicit_deadlines_reach_flutter(self):
        for configured, expected in [(None, "30s"), ("10m", "10m")]:
            with self.subTest(configured=configured):
                arguments = self.run_target(configured)
                self.assertIn("--timeout=" + expected, arguments)
                self.assertIn("--coverage", arguments)
                self.assertIn("--concurrency=2", arguments)

    def run_target(self, timeout):
        with tempfile.TemporaryDirectory(prefix="ghostr-coverage-command-") as directory:
            root = Path(directory)
            capture = root / "arguments.json"
            flutter = root / "flutter"
            flutter.write_text(
                "#!/usr/bin/env python3\nimport json, pathlib, sys\n"
                + "pathlib.Path(" + repr(str(capture))
                + ").write_text(json.dumps(sys.argv[1:]))\n"
            )
            flutter.chmod(0o755)
            command = ["make", "-s", "test-coverage", "FLUTTER=" + str(flutter),
                       "FLUTTER_TEST_CONCURRENCY=2"]
            if timeout is not None:
                command.append("FLUTTER_TEST_TIMEOUT=" + timeout)
            subprocess.run(command, cwd=ROOT, check=True, capture_output=True)
            return json.loads(capture.read_text())


if __name__ == "__main__":
    unittest.main()
