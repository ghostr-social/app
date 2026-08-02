#!/bin/sh
set -eu

apk_path=$1
dex_entries=$(unzip -Z1 "$apk_path" | awk '/^classes[0-9]*\.dex$/')

if [ -z "$dex_entries" ]; then
  echo "Release APK contains no DEX files: $apk_path" >&2
  exit 1
fi

for dex_entry in $dex_entries; do
  if unzip -p "$apk_path" "$dex_entry" \
    | strings \
    | grep -Fq 'dev/flutter/plugins/integration_test/IntegrationTestPlugin'; then
    echo "Integration-test code is packaged in release APK: $dex_entry" >&2
    exit 1
  fi
done
