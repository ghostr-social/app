#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
makefile="$root/Makefile"

if grep -Fq 'native-dead-code: native-dead-code-install' "$makefile"; then
  echo 'native-dead-code must not install or inspect user-scoped tooling' >&2
  exit 1
fi

grep -Fq 'cargo +1.97.1 hawk check --only test-only -D hawk::test_only' "$makefile"
