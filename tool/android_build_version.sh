#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <git-ref>" >&2
  exit 64
fi

ref=$1
case "$ref" in
  refs/tags/v*)
    name=${ref#refs/tags/v}
    if ! printf '%s\n' "$name" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
      echo "Release tag must look like vMAJOR.MINOR.PATCH, got: $ref" >&2
      exit 65
    fi
    major=${name%%.*}
    rest=${name#*.}
    minor=${rest%%.*}
    patch=${rest#*.}
    if [ "$minor" -gt 999 ] || [ "$patch" -gt 999 ]; then
      echo "Release minor and patch versions must be at most 999." >&2
      exit 65
    fi
    code=$((major * 1000000 + minor * 1000 + patch))
    if [ "$code" -lt 1 ]; then
      echo "Release tag must produce a positive Android versionCode." >&2
      exit 65
    fi
    ;;
  *)
    version=$(sed -n 's/^version:[[:space:]]*//p' pubspec.yaml)
    name=${version%+*}
    code=${version#*+}
    ;;
esac

echo "BUILD_NAME=$name"
echo "BUILD_NUMBER=$code"
