#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
makefile="$root/Makefile"

grep -Fq 'video-android-emulator-tests:' "$makefile"
grep -Fq 'VIDEO_ANDROID_EMULATOR_SERIAL ?= emulator-5580' "$makefile"

for test_name in \
  bandwidth_drop packet_loss high_rtt rapid_swipes held_response manifest_retry
do
  grep -Fq "integration_test/${test_name}_video_test.dart" "$makefile"
done

qoe_doc="$root/standards/VIDEO_QOE_TARGETS.md"
grep -Fq 'device-side held-response proxy' "$qoe_doc"
grep -Fq 'device-side same-URL manifest-retry proxy' "$qoe_doc"
