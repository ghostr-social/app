#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
makefile="$root/Makefile"
runner="$root/tool/run_video_android_offline_restart.sh"
output=$(mktemp)
trap 'rm -f "$output"' EXIT HUP INT TERM

if make -s -C "$root" video-android-offline-restart >"$output" 2>&1; then
  echo 'offline restart target accepted a missing serial' >&2
  exit 1
fi
grep -Fq 'Set ANDROID_PHYSICAL_SERIAL' "$output"

if make -s -C "$root" video-android-offline-restart \
  ANDROID_PHYSICAL_SERIAL=emulator-5580 >"$output" 2>&1
then
  echo 'offline restart target accepted an emulator serial' >&2
  exit 1
fi
grep -Fq 'must identify physical hardware' "$output"

grep -Fq 'video-android-offline-restart:' "$makefile"
grep -Fq 'run_video_android_offline_restart.sh' "$makefile"
grep -Fq 'adb devices -l' "$runner"
grep -Fq 'shell getprop ro.kernel.qemu' "$runner"
grep -Fq 'integration_test/warp_feed_offline_seed_video_test.dart' "$runner"
grep -Fq 'shell am force-stop app.ghostr' "$runner"
grep -Fq 'integration_test/warp_feed_offline_restore_video_test.dart' "$runner"
test "$(grep -Fc 'flutter test --no-uninstall' "$runner")" -eq 2
