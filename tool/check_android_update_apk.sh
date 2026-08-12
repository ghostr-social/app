#!/bin/sh
set -eu

stable_certificate='1e2c0712ebbc909cb2aa7ea9af97ae620639f1e01463f28f6ee1e68c1ed3b340'

if [ "$#" -ne 4 ]; then
  echo "Usage: $0 <apk> <abi> <version-name> <version-code>" >&2
  exit 64
fi

apk_path=$1
expected_abi=$2
expected_name=$3
expected_code=$4
script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)

find_sdk_tool() {
  tool_name=$1
  command -v "$tool_name" 2>/dev/null && return
  sdk_root=${ANDROID_SDK_ROOT:-${ANDROID_HOME:-}}
  test -n "$sdk_root" || return 1
  find "$sdk_root" -type f -name "$tool_name" -perm -u+x 2>/dev/null \
    | sort -V \
    | tail -n 1
}

require_tool() {
  tool_path=$(find_sdk_tool "$1")
  test -n "$tool_path" || {
    echo "Required Android SDK tool is unavailable: $1" >&2
    exit 69
  }
  printf '%s\n' "$tool_path"
}

assert_value() {
  label=$1
  expected=$2
  actual=$3
  test "$actual" = "$expected" || {
    echo "Expected $label $expected, got $actual" >&2
    exit 1
  }
}

test -s "$apk_path" || {
  echo "APK is missing or empty: $apk_path" >&2
  exit 66
}
apkanalyzer=$(require_tool apkanalyzer)
apksigner=$(require_tool apksigner)

sh "$script_dir/check_android_apk_abi.sh" "$apk_path" "$expected_abi"
sh "$script_dir/check_android_release_apk.sh" "$apk_path"

package_name=$("$apkanalyzer" manifest application-id "$apk_path")
version_name=$("$apkanalyzer" manifest version-name "$apk_path")
version_code=$("$apkanalyzer" manifest version-code "$apk_path")
assert_value package app.ghostr "$package_name"
assert_value 'version name' "$expected_name" "$version_name"
assert_value 'version code' "$expected_code" "$version_code"

certificate_output=$("$apksigner" verify --print-certs "$apk_path")
certificate=$(printf '%s\n' "$certificate_output" \
  | sed -n 's/^Signer #1 certificate SHA-256 digest: //p' \
  | head -n 1 \
  | tr '[:upper:]' '[:lower:]')
test "$certificate" = "$stable_certificate" || {
  echo "Unexpected signing certificate SHA-256: ${certificate:-missing}" >&2
  exit 1
}
