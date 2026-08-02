#!/bin/sh
set -eu

if [ "$#" -ne 4 ]; then
  echo "Usage: $0 <tag> <arm64-apk> <armeabi-v7a-apk> <x86_64-apk>" >&2
  exit 64
fi

tag=$1
arm64_apk=$2
armeabi_apk=$3
x86_64_apk=$4

if ! gh release view "$tag" >/dev/null 2>&1; then
  gh release create "$tag" \
    --title "Release $tag" \
    --generate-notes
fi
gh release upload "$tag" \
  "$arm64_apk" \
  "$armeabi_apk" \
  "$x86_64_apk" \
  --clobber
