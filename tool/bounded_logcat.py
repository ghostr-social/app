#!/usr/bin/env python3
"""Bound optional Android log capture when a phone disappears from USB."""
import argparse
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("serial")
    parser.add_argument("mode", choices=("clear", "dump"))
    parser.add_argument("--timeout", type=float, default=10)
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("timeout must be positive")
    command = ["adb", "-s", args.serial, "logcat", {"clear": "-c", "dump": "-d"}[args.mode]]
    try:
        return subprocess.run(command, timeout=args.timeout, check=False).returncode
    except subprocess.TimeoutExpired:
        print(f"Optional logcat {args.mode} timed out after {args.timeout}s", file=sys.stderr)
        return 124


if __name__ == "__main__":
    sys.exit(main())
