#!/bin/sh
set -eu

checker=tool/check_native_coverage.awk
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT

resolve_path() {
  case "$1" in
    rust/src/*) printf '%s\n' "$1" ;;
    *)
      rg --files rust/src |
        awk -F/ -v name="$1" '$NF == name { print; exit }'
      ;;
  esac
}

complete_lcov=$fixture/complete.lcov
sed -n \
  's/^[[:space:]]*threshold\["\([^"]*\)"\][[:space:]]*=.*$/\1/p' \
  "$checker" |
  while IFS= read -r required; do
    source=$(resolve_path "$required")
    test -n "$source"
    printf 'SF:%s/%s\nDA:1,1\nend_of_record\n' "$fixture" "$source"
  done >"$complete_lcov"

collision_lcov=$fixture/collision.lcov
{
  printf 'SF:%s/rust/src/synthetic/event_identity.rs\n' "$fixture"
  printf 'DA:1,0\nend_of_record\n'
  sed -n '1,$p' "$complete_lcov"
} >"$collision_lcov"

collision_output=$fixture/collision.out
failed=0

require_threshold() {
  actual=$(awk -v source="$1" \
    '$0 ~ "threshold\\[\"" source "\"\\]" { print $3 }' "$checker")
  if [ "$actual" != "$2" ]; then
    printf 'Expected native threshold %s for %s, found %s\n' \
      "$2" "$1" "${actual:-none}"
    failed=1
  fi
}

require_threshold rust/src/engine/budget.rs 100
require_threshold rust/src/engine/inventory_controller.rs 100
require_threshold rust/src/discovery/plan_executor.rs 100
require_threshold rust/src/video/delivery_retry.rs 100
require_threshold rust/src/video/partial_range_manifest.rs 100
require_threshold rust/src/video/progressive_posts.rs 100

if ! awk -f "$checker" "$collision_lcov" >"$collision_output" 2>&1; then
  printf '%s\n' 'unrelated same-basename LCOV record changed the contract result'
  sed -n '1,$p' "$collision_output"
  failed=1
fi

missing_lcov=$fixture/missing.lcov
sed 's#rust/src/video/event_identity.rs#rust/src/synthetic/event_identity.rs#' \
  "$complete_lcov" >"$missing_lcov"
missing_output=$fixture/missing.out
if awk -f "$checker" "$missing_lcov" >"$missing_output" 2>&1; then
  printf '%s\n' 'same-basename LCOV record satisfied a missing canonical path'
  failed=1
fi
if ! grep -Fq \
  'Missing native coverage record for rust/src/video/event_identity.rs' \
  "$missing_output"; then
  printf '%s\n' 'missing canonical path was not reported exactly'
  sed -n '1,$p' "$missing_output"
  failed=1
fi

exit "$failed"
