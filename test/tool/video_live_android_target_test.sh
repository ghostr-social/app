#!/bin/sh
# The live runner must preserve the installed account and reject emulators.
set -eu
root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
cd "$root"
test -f tool/run_video_live_android.sh
rg -q '^video-live-android-tests:' Makefile
rg -q '^video-live-android-evidence:' Makefile
rg -q -- '--keep-app-running' tool/run_video_live_android.sh
rg -q -- '--profile' tool/run_video_live_android.sh
rg -q 'ro.kernel.qemu' tool/run_video_live_android.sh
rg -q 'install -r' tool/run_video_live_android.sh
rg -q -- '--use-existing-app' tool/run_video_live_android.sh
if rg -n 'pm clear|adb.*uninstall|setMock|WarpFeedRelay|ProgressiveDeviceOrigin|databaseFactoryMemory' \
  tool/run_video_live_android.sh integration_test/support/live_*.dart; then
  echo 'Live tests must preserve app data and use real services.' >&2
  exit 1
fi
if sh tool/run_video_live_android.sh emulator-5580 >/dev/null 2>&1; then
  echo 'Live runner accepted an emulator.' >&2
  exit 1
fi
