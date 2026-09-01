#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
serial=${1:-}
test_path=integration_test/warp_feed_android_lifecycle_video_test.dart
output=$(mktemp)
test_status="$output.status"
test_pid=

cleanup() {
  if test -n "$test_pid" && kill -0 "$test_pid" 2>/dev/null; then
    kill "$test_pid" 2>/dev/null || true
    wait "$test_pid" 2>/dev/null || true
  fi
  rm -f "$output" "$test_status"
}
trap cleanup EXIT HUP INT TERM

wait_for_marker() {
  marker=$1
  limit=$2
  attempts=0
  until grep -Fq "$marker" "$output"; do
    if test -s "$test_status"; then
      status=$(cat "$test_status")
      wait "$test_pid" 2>/dev/null || true
      test_pid=
      echo "Flutter test exited before $marker (status $status)." >&2
      cat "$output" >&2
      if test "$status" -eq 0; then return 1; fi
      return "$status"
    fi
    attempts=$((attempts + 1))
    if test "$attempts" -ge "$limit"; then
      echo "Timed out waiting for $marker." >&2
      cat "$output" >&2
      return 1
    fi
    sleep 0.1
  done
}

report_foreground() {
  echo 'WARP_ANDROID_FOREGROUND_DIAGNOSTICS'
  adb -s "$serial" shell pidof app.ghostr 2>&1 || true
  adb -s "$serial" shell dumpsys activity activities 2>&1 |
    grep -E 'mResumedActivity|topResumedActivity|app\.ghostr/social\.ghostr\.MainActivity' || true
  adb -s "$serial" shell dumpsys window windows 2>&1 |
    grep -E 'mCurrentFocus|mFocusedApp|app\.ghostr/social\.ghostr\.MainActivity' || true
}

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
(
  flutter_pid=
  stop_flutter() {
    kill "$flutter_pid" 2>/dev/null || true
    wait "$flutter_pid" 2>/dev/null || true
    exit 143
  }
  trap stop_flutter HUP INT TERM
  status=0
  flutter test --no-uninstall --no-pub "$test_path" \
    -d "$serial" >"$output" 2>&1 &
  flutter_pid=$!
  wait "$flutter_pid" || status=$?
  flutter_pid=
  printf '%s\n' "$status" >"$test_status"
  exit "$status"
) &
test_pid=$!
wait_for_marker WARP_ANDROID_LIFECYCLE_READY 6000
adb -s "$serial" shell am start -a android.intent.action.MAIN \
  -c android.intent.category.HOME >/dev/null
wait_for_marker WARP_ANDROID_LIFECYCLE_BACKGROUND 300
adb -s "$serial" shell monkey -p app.ghostr \
  -c android.intent.category.LAUNCHER 1
report_foreground
wait_for_marker WARP_ANDROID_LIFECYCLE_RESUMED 300

status=0
wait "$test_pid" || status=$?
test_pid=
cat "$output"
exit "$status"
