#!/bin/sh
# Summarize one evidence directory produced by tool/run_warp_evidence.sh.
#
#   tool/summarize_warp_evidence.sh EVIDENCE_DIR
#
# Prints a Markdown fragment: the command, commit, device, exit status,
# per-test-file outcome parsed from flutter's progress lines, every
# WARP_* marker line, and the first-frame latencies the tests report.
set -eu

dir=${1:-}
test -d "$dir" || { echo 'usage: summarize_warp_evidence.sh EVIDENCE_DIR' >&2; exit 64; }

printf '## Run `%s`\n\n' "$(basename "$dir")"
printf -- '- command: `%s`\n' "$(cat "$dir/command.txt")"
printf -- '- commit: `%s` (%s)\n' "$(sed -n 1p "$dir/commit.txt")" "$(sed -n 3p "$dir/commit.txt")"
printf -- '- device: %s\n' "$(grep -E '^ro\.' "$dir/device.txt" | tr '\n' ' ' 2>/dev/null || echo none)"
printf -- '- exit: %s\n\n' "$(cat "$dir/exit.txt")"

if test -s "$dir/stdout.log"; then
  printf '| test file | last progress | result |\n|---|---|---|\n'
  tr '\r' '\n' <"$dir/stdout.log" | awk '
    function counter(   c) { c = $2; if ($3 ~ /^-[0-9]+:?$/) c = c " " $3; sub(/:$/, "", c); return c }
    function failures(c) { return (c ~ /-/) ? substr(c, index(c, "-") + 1) + 0 : 0 }
    function passes(c) { return substr(c, 2) + 0 }
    function verdict(start, finish) {
      if (failures(finish) > failures(start)) return "FAIL"
      if (passes(finish) > passes(start)) return "pass"
      return "not run (stopped)"
    }
    /^[0-9]+:[0-9]+ \+[0-9]+/ {
      cur = counter()
      if ($0 ~ /: loading .*integration_test\//) {
        match($0, /integration_test\/[a-z0-9_]+\.dart/); next_file = substr($0, RSTART, RLENGTH)
        if (next_file != file) {
          if (file != "") print "| " file " | " cur " | " verdict(start, cur) " |"
          file = next_file; start = cur
        }
      }
      last = cur
    }
    END { if (file != "") print "| " file " | " last " | " verdict(start, last) " |" }'
  printf '\n'
fi

if grep -aq 'WARP_QOE' "$dir/stdout.log" 2>/dev/null; then
  printf '### Swipe-to-first-frame samples (WARP_QOE lines, all test files in this run)\n\n'
  printf '| metric | n | min | p50 | p95 | max | unit |\n|---|---|---|---|---|---|---|\n'
  for metric in startup_ms focus_switch_ms native_frame_ms presented_ms rust_ready_ms; do
    tr '\r' '\n' <"$dir/stdout.log" | grep -a 'WARP_QOE' |
      grep -oE "(^|[[:space:]])$metric=-?[0-9]+" | sed -E 's/.*=//' | sort -n |
      awk -v m="$metric" '{ v[NR] = $1 } END {
        if (NR == 0) next
        p50 = v[int((NR + 1) / 2)]; p95 = v[int(NR * 0.95) < 1 ? 1 : int(NR * 0.95)]
        printf "| %s | %d | %d | %d | %d | %d | ms |\n", m, NR, v[1], p50, p95, v[NR] }'
  done
  printf '\nPercentiles are nearest-rank over every sample printed by the integration tests in this\n'
  printf 'run; negative values mean the item was ready before the swipe (prepared reserve).\n\n'
fi

if test -s "$dir/markers.log"; then
  printf '### WARP markers\n\n```\n'
  tr '\r' '\n' <"$dir/markers.log" | grep -aoE 'WARP_[A-Z_]+.*' | grep -avE '^WARP_QOE|^WARP_PLAN |^WARP_PREPARATION ' | head -80
  printf '```\n\n'
fi

if grep -qiE 'first[-_ ]?frame' "$dir/stdout.log" 2>/dev/null; then
  printf '### First-frame lines\n\n```\n'
  tr '\r' '\n' <"$dir/stdout.log" | grep -iE 'first[-_ ]?frame' | grep -vE 'loading|^\s*$' | cut -c1-160 | head -40
  printf '```\n'
fi
