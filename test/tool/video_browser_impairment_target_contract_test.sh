#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
makefile="$root/Makefile"

grep -Fq 'video-user-e2e-impairments:' "$makefile"
grep -Fq 'VIDEO_BROWSER_SCENARIOS := adaptive_plans $(VIDEO_IMPAIRMENT_SCENARIOS)' "$makefile"
grep -Fq 'for scenario in $(VIDEO_BROWSER_SCENARIOS)' "$makefile"
grep -Fq -- '--scenario="$$scenario"' "$makefile"

for scenario in \
  adaptive_plans bandwidth_drop packet_loss high_rtt rapid_swipes storage_pressure source_failure \
  protected_transitions
do
  grep -Fq "$scenario" "$makefile"
done
