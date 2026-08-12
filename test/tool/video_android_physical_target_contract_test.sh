#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
makefile="$root/Makefile"
output=$(mktemp)
trap 'rm -f "$output"' EXIT HUP INT TERM

if make -s -C "$root" video-android-physical-tests >"$output" 2>&1; then
  echo 'physical target accepted a missing serial' >&2
  exit 1
fi
grep -Fq 'Set ANDROID_PHYSICAL_SERIAL' "$output"

if make -s -C "$root" video-android-physical-tests \
  ANDROID_PHYSICAL_SERIAL=emulator-5580 >"$output" 2>&1
then
  echo 'physical target accepted an emulator serial' >&2
  exit 1
fi
grep -Fq 'must identify physical hardware' "$output"
grep -Fq 'ro.kernel.qemu' "$makefile"
