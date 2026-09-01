#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
runner="$root/tool/run_video_android_lifecycle.sh"
temp_root=/private/tmp
if ! test -d "$temp_root" || ! test -w "$temp_root"; then temp_root=/tmp; fi
fixture=$(TMPDIR="$temp_root" mktemp -d)
runner_pid=

cleanup() {
  if test -n "$runner_pid" && kill -0 "$runner_pid" 2>/dev/null; then
    kill "$runner_pid" 2>/dev/null || true
    wait "$runner_pid" 2>/dev/null || true
  fi
  rm -rf "$fixture"
}
trap cleanup EXIT HUP INT TERM
mkdir -p "$fixture/bin"
runner_status="$fixture/runner-status"

printf '%s\n' '#!/bin/sh' \
  'case "$*" in' \
  '  "devices -l") echo "physical device usb:test model:test" ;;' \
  '  *"get-state") echo device ;;' \
  '  *"getprop ro.kernel.qemu") exit 0 ;;' \
  '  *) exit 0 ;;' \
  'esac' >"$fixture/bin/adb"
printf '%s\n' '#!/bin/sh' \
  'echo INSTALL_FAILED_USER_RESTRICTED' \
  'exit 42' >"$fixture/bin/flutter"
chmod +x "$fixture/bin/adb" "$fixture/bin/flutter"

(
  child_pid=
  stop_child() {
    kill "$child_pid" 2>/dev/null || true
    wait "$child_pid" 2>/dev/null || true
    exit 143
  }
  trap stop_child HUP INT TERM
  status=0
  TMPDIR="$temp_root" PATH="$fixture/bin:$PATH" sh "$runner" physical \
    >"$fixture/output" 2>&1 &
  child_pid=$!
  wait "$child_pid" || status=$?
  child_pid=
  printf '%s\n' "$status" >"$runner_status"
  exit "$status"
) &
runner_pid=$!
attempts=0
while ! test -s "$runner_status" && test "$attempts" -lt 30; do
  attempts=$((attempts + 1))
  sleep 0.1
done
if ! test -s "$runner_status"; then
  cat "$fixture/output" >&2
  echo 'lifecycle runner ignored an exited Flutter child' >&2
  exit 1
fi
status=$(cat "$runner_status")
wait "$runner_pid" 2>/dev/null || true
runner_pid=
test "$status" -ne 0
grep -Fq 'INSTALL_FAILED_USER_RESTRICTED' "$fixture/output"
grep -Fq 'Flutter test exited before WARP_ANDROID_LIFECYCLE_READY' \
  "$fixture/output"
