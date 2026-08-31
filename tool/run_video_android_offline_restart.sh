#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
serial=${1:-}
seed=integration_test/warp_feed_offline_seed_video_test.dart
restore=integration_test/warp_feed_offline_restore_video_test.dart

test -n "$serial" || { echo 'Android physical serial is required.' >&2; exit 1; }
case "$serial" in emulator-*)
  echo 'Android serial must identify physical hardware.' >&2
  exit 1
esac

adb devices -l
state=$(adb -s "$serial" get-state 2>/dev/null || true)
test "$state" = device || { echo "Android device $serial is not ready." >&2; exit 1; }
raw_qemu=$(adb -s "$serial" shell getprop ro.kernel.qemu)
qemu=$(printf '%s' "$raw_qemu" | tr -d '\r')
test "$qemu" != 1 || { echo 'Android serial must identify physical hardware.' >&2; exit 1; }

cd "$root"
flutter test --no-uninstall --no-pub "$seed" -d "$serial"
adb -s "$serial" shell am force-stop app.ghostr
flutter test --no-uninstall --no-pub "$restore" -d "$serial"
