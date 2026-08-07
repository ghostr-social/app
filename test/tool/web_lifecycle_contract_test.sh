#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
launcher="$root/tool/run_video_debug_web.sh"
sandbox=$(mktemp -d)
first_runner=

cleanup() {
  if [ -n "$first_runner" ]; then
    kill "$first_runner" 2>/dev/null || true
  fi
  rm -rf "$sandbox"
}

wait_for_file() {
  file=$1
  attempts=0
  while [ ! -f "$file" ] && [ "$attempts" -lt 100 ]; do
    sleep 0.02
    attempts=$((attempts + 1))
  done
  test -f "$file"
}

trap cleanup EXIT
mkdir -p "$sandbox/bin" "$sandbox/rust/target/debug"
printf '#!/bin/sh\nexit 0\n' > "$sandbox/bin/cargo"
printf '#!/bin/sh\nprintf "sh %%s\\n" "$WEB_TEST_LAUNCHER"\n' > "$sandbox/bin/ps"
printf '%s\n' '#!/bin/sh' \
  'if [ "$WEB_TEST_ROLE" = first ]; then' \
  '  touch "$GHOSTR_VIDEO_DEBUG_CACHE/old-event" "$GHOSTR_VIDEO_DEBUG_CACHE/old-video"' \
  '  printf "%s\n" "$GHOSTR_VIDEO_DEBUG_CACHE" > "$WEB_TEST_FIRST_CACHE"' \
  '  touch "$WEB_TEST_FIRST_READY"' \
  '  trap '"'"'touch "$WEB_TEST_FIRST_STOPPED"; exit 0'"'"' HUP INT TERM' \
  '  while :; do sleep 1; done' \
  'fi' \
  'find "$GHOSTR_VIDEO_DEBUG_CACHE" -mindepth 1 -print -quit > "$WEB_TEST_CONTENTS"' \
  'printf "%s\n" "$GHOSTR_VIDEO_DEBUG_CACHE" > "$WEB_TEST_SECOND_CACHE"' \
  > "$sandbox/rust/target/debug/video-debug"
chmod +x "$sandbox/bin/cargo" "$sandbox/bin/ps"
chmod +x "$sandbox/rust/target/debug/video-debug"

PATH="$sandbox/bin:$PATH" WEB_TEST_ROLE=first \
  WEB_TEST_LAUNCHER="$launcher" \
  WEB_TEST_FIRST_CACHE="$sandbox/first-cache" \
  WEB_TEST_FIRST_READY="$sandbox/first-ready" \
  WEB_TEST_FIRST_STOPPED="$sandbox/first-stopped" \
  sh "$launcher" "$sandbox/video-debug-cache" "$sandbox/state" "$sandbox/rust" &
first_runner=$!
wait_for_file "$sandbox/first-ready"
first_cache=$(cat "$sandbox/first-cache")

PATH="$sandbox/bin:$PATH" WEB_TEST_ROLE=second \
  WEB_TEST_LAUNCHER="$launcher" \
  WEB_TEST_CONTENTS="$sandbox/second-contents" \
  WEB_TEST_SECOND_CACHE="$sandbox/second-cache" \
  sh "$launcher" "$sandbox/video-debug-cache" "$sandbox/state" "$sandbox/rust"

wait_for_file "$sandbox/first-stopped"
if kill -0 "$first_runner" 2>/dev/null; then
  echo "the previous web debugger is still running" >&2
  exit 1
fi
first_runner=
second_cache=$(cat "$sandbox/second-cache")
test ! -e "$first_cache"
test ! -e "$second_cache"
test ! -s "$sandbox/second-contents"
test ! -e "$sandbox/state/owner"
