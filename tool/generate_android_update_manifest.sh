#!/bin/sh
set -eu

if [ "$#" -ne 5 ]; then
  echo "Usage: $0 <tag> <published-at> <arm64-apk> <armeabi-v7a-apk> <x86_64-apk>" >&2
  exit 64
fi

tag=$1
published_at=$2
arm64_apk=$3
armeabi_apk=$4
x86_64_apk=$5
script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)

version_output=$(sh "$script_dir/android_build_version.sh" "refs/tags/$tag")
version_name=$(printf '%s\n' "$version_output" | sed -n 's/^BUILD_NAME=//p')
version_code=$(printf '%s\n' "$version_output" | sed -n 's/^BUILD_NUMBER=//p')

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

artifact_json() {
  abi=$1
  file=$2
  comma=$3
  test -s "$file" || { echo "APK is missing or empty: $file" >&2; exit 66; }
  name=$(basename "$file")
  expected="ghostr-$tag-$abi.apk"
  test "$name" = "$expected" || {
    echo "Expected $expected, got $name" >&2
    exit 65
  }
  size=$(wc -c < "$file" | tr -d '[:space:]')
  digest=$(sha256_file "$file")
  url="https://github.com/ghostr-social/app/releases/download/$tag/$name"
  printf '    {"abi":"%s","url":"%s","size":%s,"sha256":"%s"}%s\n' \
    "$abi" "$url" "$size" "$digest" "$comma"
}

printf '%s\n' '{'
printf '  "schemaVersion":1,\n'
printf '  "namespace":"ghostr.social",\n'
printf '  "packageName":"app.ghostr",\n'
printf '  "channel":"stable",\n'
printf '  "versionName":"%s",\n' "$version_name"
printf '  "versionCode":%s,\n' "$version_code"
printf '  "publishedAt":"%s",\n' "$published_at"
printf '  "releaseUrl":"https://github.com/ghostr-social/app/releases/tag/%s",\n' "$tag"
printf '  "artifacts":[\n'
artifact_json 'arm64-v8a' "$arm64_apk" ','
artifact_json 'armeabi-v7a' "$armeabi_apk" ','
artifact_json 'x86_64' "$x86_64_apk" ''
printf '  ]\n'
printf '%s\n' '}'
