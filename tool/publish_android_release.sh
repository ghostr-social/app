#!/bin/sh
set -eu

if [ "$#" -ne 6 ]; then
  echo "Usage: $0 <tag> <arm64-apk> <armeabi-v7a-apk> <x86_64-apk> <manifest> <target>" >&2
  exit 64
fi

tag=$1
arm64_apk=$2
armeabi_apk=$3
x86_64_apk=$4
manifest=$5
target=$6
: "${GH_REPO:?GH_REPO must identify the release repository}"

resolve_tag_commit() {
  gh api "repos/$GH_REPO/commits/$tag" --jq .sha
}

require_built_commit() {
  resolved=$1
  if [ "$resolved" != "$target" ]; then
    echo "Release tag $tag resolves to $resolved and does not match built commit $target." >&2
    exit 65
  fi
}

tag_commit=''
if tag_commit=$(resolve_tag_commit 2>/dev/null); then
  require_built_commit "$tag_commit"
fi

if ! gh release view "$tag" >/dev/null 2>&1; then
  gh release create "$tag" \
    --title "Release $tag" \
    --generate-notes \
    --target "$target" \
    --draft
fi
if [ -z "$tag_commit" ]; then
  tag_commit=$(resolve_tag_commit) || {
    echo "Could not resolve release tag $tag after creation." >&2
    exit 69
  }
  require_built_commit "$tag_commit"
fi
gh release upload "$tag" \
  "$arm64_apk" \
  "$armeabi_apk" \
  "$x86_64_apk" \
  "$manifest"
gh release edit "$tag" --draft=false
