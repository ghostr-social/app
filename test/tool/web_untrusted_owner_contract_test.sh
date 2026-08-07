#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
launcher="$root/tool/run_video_debug_web.sh"
sandbox=$(mktemp -d)
unrelated=

cleanup() {
  if [ -n "$unrelated" ]; then
    kill "$unrelated" 2>/dev/null || true
    wait "$unrelated" 2>/dev/null || true
  fi
  rm -rf "$sandbox"
}

trap cleanup EXIT
mkdir -p "$sandbox/bin" "$sandbox/rust/target/debug" "$sandbox/state"
sleep 30 &
unrelated=$!
printf '%s\n' "$unrelated" > "$sandbox/state/owner"
printf '#!/bin/sh\nexit 0\n' > "$sandbox/bin/cargo"
printf '#!/bin/sh\nexit 0\n' > "$sandbox/rust/target/debug/video-debug"
printf '#!/bin/sh\nprintf "sleep 30\\n"\n' > "$sandbox/bin/ps"
chmod +x "$sandbox/bin/cargo" "$sandbox/bin/ps"
chmod +x "$sandbox/rust/target/debug/video-debug"

PATH="$sandbox/bin:$PATH" sh "$launcher" \
  "$sandbox/video-debug-cache" "$sandbox/state" "$sandbox/rust"

if ! kill -0 "$unrelated" 2>/dev/null; then
  echo "an unrelated process recorded by a stale owner was killed" >&2
  exit 1
fi
test ! -e "$sandbox/state/owner"
