#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <git-ref>" >&2
  exit 64
fi

ref=$1
case "$ref" in
  refs/tags/v*)
    printf '%s\n' "${ref#refs/tags/}"
    exit 0
    ;;
esac

latest=$(git tag --list | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -n 1)
if [ -z "$latest" ]; then
  echo "v0.0.1"
  exit 0
fi

version=${latest#v}
major=${version%%.*}
rest=${version#*.}
minor=${rest%%.*}
patch=${rest#*.}
echo "v$major.$minor.$((patch + 1))"
