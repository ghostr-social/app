#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
makefile="$root/Makefile"
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

grep -Fq 'video-android-lifecycle:' "$makefile"
grep -Fq 'run_video_android_lifecycle.sh' "$makefile"
grep -Fq 'adb devices -l' "$runner"
grep -Fq 'shell getprop ro.kernel.qemu' "$runner"
grep -Fq 'integration_test/warp_feed_android_lifecycle_video_test.dart' "$runner"
grep -Fq 'flutter test --no-uninstall --no-pub' "$runner"
grep -Fq 'android.intent.action.MAIN' "$runner"
grep -Fq 'android.intent.category.HOME' "$runner"
grep -Fq 'shell monkey -p app.ghostr' "$runner"
grep -Fq 'android.intent.category.LAUNCHER' "$runner"
grep -Fq 'android.intent.category.LAUNCHER 1' "$runner"
if grep -Fq 'shell am start -W --user current' "$runner"; then
  echo 'lifecycle target retained ineffective MIUI foreground command' >&2
  exit 1
fi
grep -Fq 'app\.ghostr/social\.ghostr\.MainActivity' "$runner"
grep -Fq 'dumpsys activity activities' "$runner"
grep -Fq 'dumpsys window windows' "$runner"
grep -Fq 'mCurrentFocus' "$runner"
grep -Fq 'WARP_ANDROID_LIFECYCLE_READY' "$runner"
grep -Fq 'WARP_ANDROID_LIFECYCLE_BACKGROUND' "$runner"
grep -Fq 'WARP_ANDROID_LIFECYCLE_RESUMED' "$runner"
grep -Fq '$(MAKE) video-android-lifecycle' "$makefile"
grep -Fq 'ANDROID_PHYSICAL_SERIAL="$(ANDROID_PHYSICAL_SERIAL)"' "$makefile"
