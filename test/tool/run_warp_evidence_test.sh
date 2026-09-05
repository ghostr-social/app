#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
evidence=$(mktemp -d)
trap 'rm -rf "$evidence"' EXIT HUP INT TERM

set +e
WARP_EVIDENCE_ROOT="$evidence" sh "$root/tool/run_warp_evidence.sh" none contract \
  sh -c 'echo "WARP_QOE p50=1"; echo plain; exit 3' >/dev/null
status=$?
set -e
test "$status" -eq 3 || { echo "expected exit 3, got $status" >&2; exit 1; }

dir=$(ls -d "$evidence"/*-none-contract)
test "$(cat "$dir/exit.txt")" = 3
grep -Fq 'WARP_QOE p50=1' "$dir/markers.log"
if grep -Fq plain "$dir/markers.log"; then
  echo 'markers.log must contain only WARP_ lines' >&2
  exit 1
fi
grep -Fq plain "$dir/stdout.log"
grep -Fq 'sh -c' "$dir/command.txt"
grep -Eq '^[0-9a-f]{40}$' "$dir/commit.txt"
grep -Fq 'exit=3' "$dir/summary.txt"
grep -Fq 'no device' "$dir/device.txt"
grep -Eq '^[0-9a-f]{64}$' "$dir/source-before.sha256"
cmp "$dir/source-before.sha256" "$dir/source-after.sha256"

set +e
sh "$root/tool/run_warp_evidence.sh" none >/dev/null 2>&1
usage=$?
set -e
test "$usage" -eq 64 || { echo "expected usage exit 64, got $usage" >&2; exit 1; }
