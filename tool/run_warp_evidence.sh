#!/bin/sh
# Run a command and retain durable evidence of the run.
#
#   tool/run_warp_evidence.sh SERIAL LABEL COMMAND [ARG...]
#
# SERIAL is an adb serial, or `none` when no device is involved. LABEL names
# the run. Evidence lands in $WARP_EVIDENCE_ROOT (default evidence/warp) under
# <utc-stamp>-<commit>-<serial>-<label>/ as: command.txt, commit.txt,
# device.txt, exit.txt, markers.log (every WARP_* line), summary.txt,
# stdout.log (full output, git-ignored) and logcat.log (git-ignored).
# The script exits with the command's own status.
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
serial=${1:-}
label=${2:-}
if test -z "$serial" || test -z "$label" || test $# -lt 3; then
  echo 'usage: run_warp_evidence.sh SERIAL LABEL COMMAND [ARG...]' >&2
  exit 64
fi
shift 2

evidence_root=${WARP_EVIDENCE_ROOT:-$root/evidence/warp}
stamp=$(date -u +%Y%m%dT%H%M%SZ)
commit=$(git -C "$root" rev-parse --short=10 HEAD)
dir="$evidence_root/$stamp-$commit-$serial-$label"
mkdir -p "$dir"

printf '%s ' "$@" >"$dir/command.txt"
printf '\n' >>"$dir/command.txt"
{
  git -C "$root" rev-parse HEAD
  git -C "$root" rev-parse --abbrev-ref HEAD
  printf 'dirty_files=%s\n' "$(git -C "$root" status --porcelain | wc -l | tr -d ' ')"
} >"$dir/commit.txt"

if test "$serial" != none; then
  {
    adb devices -l
    for prop in ro.product.model ro.build.version.release ro.kernel.qemu; do
      printf '%s=%s\n' "$prop" "$(adb -s "$serial" shell getprop "$prop" | tr -d '\r')"
    done
  } >"$dir/device.txt" 2>&1 || true
  adb -s "$serial" logcat -c >/dev/null 2>&1 || true
else
  printf 'no device\n' >"$dir/device.txt"
fi

status_file="$dir/exit.txt"
( set +e; "$@" 2>&1; printf '%s\n' "$?" >"$status_file" ) | tee "$dir/stdout.log"
status=$(cat "$status_file")

if test "$serial" != none; then
  adb -s "$serial" logcat -d 2>/dev/null |
    grep -E 'ghostr|flutter|ExoPlayer|MediaCodec|AndroidRuntime|Decoder' \
      >"$dir/logcat.log" || true
fi
grep -E 'WARP_[A-Z_]+' "$dir/stdout.log" >"$dir/markers.log" || true
{
  printf 'exit=%s\n' "$status"
  grep -E 'All tests passed|Some tests failed|Test failed|Error:' "$dir/stdout.log" | tail -20
} >"$dir/summary.txt" || true

echo "WARP_EVIDENCE_DIR=$dir"
exit "$status"
