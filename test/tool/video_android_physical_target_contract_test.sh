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
grep -Fq 'Unable to verify physical hardware' "$makefile"
grep -Fq 'VIDEO_ANDROID_PHYSICAL_TESTS :=' "$makefile"
grep -Fq 'test --no-uninstall $(VIDEO_ANDROID_PHYSICAL_TESTS)' "$makefile"
grep -Fq -- '-d "$(ANDROID_PHYSICAL_SERIAL)"' "$makefile"
grep -Fq 'integration_test/warp_feed_rapid_swipe_instrumentation_video_test.dart' "$makefile"
grep -Fq 'integration_test/warp_feed_adaptive_warm_back_video_test.dart' "$makefile"
grep -Fq 'integration_test/warp_feed_startup_singleflight_video_test.dart' "$makefile"
grep -Fq 'integration_test/warp_feed_player_verified_rescue_video_test.dart' "$makefile"
grep -Fq 'integration_test/warp_feed_bandwidth_recovery_video_test.dart' "$makefile"
grep -Fq 'integration_test/warp_feed_mixed_hls_readiness_video_test.dart' "$makefile"
grep -Fq '$(MAKE) video-android-offline-restart' "$makefile"
grep -Fq 'ANDROID_PHYSICAL_SERIAL="$(ANDROID_PHYSICAL_SERIAL)"' "$makefile"
