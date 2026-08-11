#!/bin/sh
set -eu

checker=tool/check_native_coverage.awk
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT
failed=0

thresholds=$fixture/thresholds
sed -n \
  's/^[[:space:]]*threshold\["\([^"]*\)"\][[:space:]]*=.*$/\1/p' \
  "$checker" >"$thresholds"

while IFS= read -r required; do
  if ! test -f "$required"; then
    printf 'Native coverage threshold targets missing source: %s\n' "$required"
    failed=1
  fi
done <"$thresholds"
[ "$failed" -eq 0 ] || exit "$failed"

complete_lcov=$fixture/complete.lcov
sed -n '1,$p' "$thresholds" |
  while IFS= read -r required; do
    printf 'SF:%s/%s\nDA:1,1\nend_of_record\n' "$fixture" "$required"
  done >"$complete_lcov"

collision_lcov=$fixture/collision.lcov
{
  printf 'SF:%s/rust/src/synthetic/catalog.rs\n' "$fixture"
  printf 'DA:1,0\nend_of_record\n'
  sed -n '1,$p' "$complete_lcov"
} >"$collision_lcov"

collision_output=$fixture/collision.out

require_threshold() {
  actual=$(awk -v source="$1" \
    '$0 ~ "threshold\\[\"" source "\"\\]" { print $3 }' "$checker")
  if [ "$actual" != "$2" ]; then
    printf 'Expected native threshold %s for %s, found %s\n' \
      "$2" "$1" "${actual:-none}"
    failed=1
  fi
}

require_threshold rust/crates/engine/src/budget.rs 100
require_threshold rust/crates/engine/src/concurrency.rs 100
require_threshold rust/crates/delivery/src/manager/retry.rs 100
require_threshold rust/crates/gateway/src/progressive/capabilities.rs 99
require_threshold rust/crates/partial-store/src/partial_range_store/representation.rs 99
require_threshold rust/src/api/focus_control.rs 95

if ! awk -f "$checker" "$collision_lcov" >"$collision_output" 2>&1; then
  printf '%s\n' 'unrelated same-basename LCOV record changed the contract result'
  sed -n '1,$p' "$collision_output"
  failed=1
fi

missing_lcov=$fixture/missing.lcov
sed 's#rust/crates/engine/src/catalog.rs#rust/src/synthetic/catalog.rs#' \
  "$complete_lcov" >"$missing_lcov"
missing_output=$fixture/missing.out
if awk -f "$checker" "$missing_lcov" >"$missing_output" 2>&1; then
  printf '%s\n' 'same-basename LCOV record satisfied a missing canonical path'
  failed=1
fi
if ! grep -Fq \
  'Missing native coverage record for rust/crates/engine/src/catalog.rs' \
  "$missing_output"; then
  printf '%s\n' 'missing canonical path was not reported exactly'
  sed -n '1,$p' "$missing_output"
  failed=1
fi

exit "$failed"
