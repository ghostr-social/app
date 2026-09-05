#!/usr/bin/env python3
"""Fingerprint the build inputs and tests, including uncommitted source files."""
import hashlib
from pathlib import Path
import subprocess
import sys


SOURCE_PATHS = (
    "lib", "rust", "android", "rust_builder", "assets", "integration_test",
    "test", "tool", ".cargo", "pubspec.yaml", "pubspec.lock", "Makefile",
    "WARP-v3-final.md",
)


def fingerprint(root: Path) -> str:
    names = subprocess.check_output([
        "git", "-C", str(root), "ls-files", "-cz", "--others",
        "--exclude-standard", "--", *SOURCE_PATHS,
    ])
    digest = hashlib.sha256(b"ghostr-warp-source-v1\0")
    for name in sorted(set(names.split(b"\0")) - {b""}):
        path = root / name.decode("utf-8")
        digest.update(name + b"\0")
        if not path.is_file():
            digest.update(b"deleted\0")
            continue
        digest.update(str(path.stat().st_mode & 0o111).encode() + b"\0")
        with path.open("rb") as source:
            for block in iter(lambda: source.read(65_536), b""):
                digest.update(block)
        digest.update(b"\0")
    return digest.hexdigest()


if __name__ == "__main__":
    print(fingerprint(Path(sys.argv[1])))
