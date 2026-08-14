#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
serial=emulator-5580
log=$(mktemp)
started=false
CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_DEV_DEBUG

cleanup() {
  if [ "$started" = true ]; then
    adb -s "$serial" emu kill >/dev/null 2>&1 || true
  fi
  rm -f "$log"
}
trap cleanup EXIT HUP INT TERM

cd "$root"
make android-agent-avd-create
if ! adb -s "$serial" get-state >/dev/null 2>&1; then
  make android-agent-avd-run >"$log" 2>&1 &
  emulator_pid=$!
  started=true
fi

attempt=0
while [ "$(adb -s "$serial" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" != 1 ]
do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 120 ]; then
    cat "$log" >&2
    echo 'Android contract emulator did not boot.' >&2
    exit 1
  fi
  if [ "$started" = true ] && ! kill -0 "$emulator_pid" 2>/dev/null; then
    cat "$log" >&2
    echo 'Android contract emulator exited before boot.' >&2
    exit 1
  fi
  sleep 2
done

flutter test --no-pub "$@" -d "$serial"
