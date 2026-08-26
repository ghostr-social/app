#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
makefile="$root/Makefile"

grep -Fq 'WARP_LAB_ROUTE ?= /warp' "$makefile"
grep -Fq 'WARP_LAB_TARGET := tool/warp_lab/main.dart' "$makefile"
grep -Fq 'warp-lab-contract-test:' "$makefile"
grep -Fq 'sh test/tool/warp_lab_target_contract_test.sh' "$makefile"
grep -Fq 'warp-lab-android: android-agent-avd-create warp-lab-contract-test' "$makefile"
grep -Fq -- '--target "$(WARP_LAB_TARGET)"' "$makefile"
grep -Fq -- '--route "$(WARP_LAB_ROUTE)"' "$makefile"
grep -Fq -- '-d "$(VIDEO_ANDROID_EMULATOR_SERIAL)"' "$makefile"
grep -Fq 'case "$(WARP_LAB_ROUTE)"' "$makefile"
grep -Fq -- '--target lib/main.dart' "$makefile"
