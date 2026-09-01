#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
runner="$root/tool/run_video_android_lifecycle.sh"
output=$(mktemp)
trap 'rm -f "$output"' EXIT HUP INT TERM

if make -s -C "$root" video-android-lifecycle >"$output" 2>&1; then
  echo 'lifecycle target accepted a missing serial' >&2
  exit 1
fi
grep -Fq 'Set ANDROID_PHYSICAL_SERIAL' "$output"

if make -s -C "$root" video-android-lifecycle \
  ANDROID_PHYSICAL_SERIAL=emulator-5580 >"$output" 2>&1
then
  echo 'lifecycle target accepted an emulator serial' >&2
  exit 1
fi
grep -Fq 'must identify physical hardware' "$output"

if grep -Fq 'shell am start -W --user current' "$runner"; then
  echo 'lifecycle target retained ineffective MIUI foreground command' >&2
  exit 1
fi
