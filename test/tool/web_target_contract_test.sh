#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
recipe=$(make -n -C "$root" web)

printf '%s\n' "$recipe" | grep -q 'run_video_debug_web.sh'
if printf '%s\n' "$recipe" | grep -Eiq 'flutter|dart'; then
  echo "make web must not invoke Flutter or Dart" >&2
  exit 1
fi

sandbox=$(mktemp -d)
trap 'rm -rf "$sandbox"' EXIT
cache="$sandbox/video-debug-cache"
mkdir -p "$sandbox/bin" "$sandbox/rust/target/debug" "$cache" "$cache.abandoned"
touch "$cache/stale.part" "$cache/stale.complete"
touch "$cache.abandoned/stale.part"
printf '#!/bin/sh\nexit 0\n' > "$sandbox/bin/cargo"
printf '#!/bin/sh\nprintf "%%s\\n" "$GHOSTR_VIDEO_DEBUG_CACHE" > "$WEB_TEST_CAPTURE"\nfind "$GHOSTR_VIDEO_DEBUG_CACHE" -mindepth 1 -print -quit > "$WEB_TEST_CONTENTS"\n' \
  > "$sandbox/rust/target/debug/video-debug"
chmod +x "$sandbox/bin/cargo" "$sandbox/rust/target/debug/video-debug"

PATH="$sandbox/bin:$PATH" WEB_TEST_CAPTURE="$sandbox/cache-path" \
  WEB_TEST_CONTENTS="$sandbox/cache-contents" \
  make -s -C "$root" web WEB_DEBUG_CACHE_DIR="$cache" \
    WEB_DEBUG_STATE_DIR="$sandbox/state" WEB_DEBUG_RUST_DIR="$sandbox/rust"

run_cache=$(cat "$sandbox/cache-path")
test ! -e "$cache"
test ! -e "$cache.abandoned"
test ! -s "$sandbox/cache-contents"
test "$run_cache" != "$cache"
case "$run_cache" in
  "$cache".*) ;;
  *) exit 1 ;;
esac
test ! -e "$run_cache"
test ! -e "$sandbox/state/owner"
