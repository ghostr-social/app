#!/bin/sh
set -eu

apk_path=$1
target_abi=$2

unzip -Z1 "$apk_path" | awk -F/ -v target="$target_abi" '
  $1 == "lib" && $2 != "" && $2 != target {
    print "Unexpected packaged ABI: " $2 > "/dev/stderr"
    invalid = 1
  }
  $0 == "lib/" target "/librust_lib_ghostr.so" {
    ghostr = 1
  }
  END {
    if (!ghostr) {
      print "Ghostr Rust library is missing for " target > "/dev/stderr"
    }
    exit invalid || !ghostr
  }
'
