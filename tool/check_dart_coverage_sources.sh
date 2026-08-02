#!/bin/sh
set -eu

lcov_path=$1
source_root=${2%/}
exclusions_path=$3
source_label=$(basename "$source_root")
source_list=$(mktemp)
trap 'rm -f "$source_list"' EXIT

find "$source_root" -type f -name '*.dart' -print \
  | sed "s#^$source_root/#$source_label/#" \
  | sort > "$source_list"

awk '
  FILENAME == ARGV[1] && /^SF:/ {
    file = substr($0, 4)
    sub(/^.*\/lib\//, "lib/", file)
    covered[file] = 1
    next
  }
  FILENAME == ARGV[2] {
    sub(/[[:space:]]*#.*/, "")
    if ($0 != "") excluded[$0] = 1
    next
  }
  FILENAME == ARGV[3] {
    source = $0
    if (source ~ /^lib\/src\/rust\//) next
    if (!(source in covered) && !(source in excluded)) {
      print "Dart source missing from coverage: " source > "/dev/stderr"
      failed = 1
    }
  }
  END {
    if (!failed) print "All executable Dart sources are represented in coverage"
    exit failed
  }
' "$lcov_path" "$exclusions_path" "$source_list"
