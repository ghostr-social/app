#!/bin/sh
set -eu

cache_root=${1:?cache root is required}
lifecycle_dir=${2:?lifecycle directory is required}
rust_dir=${3:?Rust directory is required}
launcher_path=$(CDPATH= cd -- "$(dirname "$0")" && pwd)/$(basename "$0")
binary_path="$rust_dir/target/debug/video-debug"
owner_file="$lifecycle_dir/owner"
run_cache=
child_pid=

valid_pid() {
  case ${1:-} in
    ''|*[!0-9]*) return 1 ;;
    *) test "$1" -gt 1 ;;
  esac
}

owner_field() {
  awk -v field="$1" 'NR == 1 { print $field }' "$owner_file"
}

wait_until_stopped() {
  pid=$1
  attempts=0
  while kill -0 "$pid" 2>/dev/null && [ "$attempts" -lt 50 ]; do
    sleep 0.02
    attempts=$((attempts + 1))
  done
  ! kill -0 "$pid" 2>/dev/null
}

stop_pid() {
  pid=${1:-}
  valid_pid "$pid" || return 0
  kill -0 "$pid" 2>/dev/null || return 0
  kill -TERM "$pid" 2>/dev/null || true
  wait_until_stopped "$pid" || kill -KILL "$pid" 2>/dev/null || true
}

process_matches() {
  pid=$1
  expected=$2
  command=$(ps -ww -p "$pid" -o command= 2>/dev/null) || return 1
  case "$command" in
    *"$expected"*) return 0 ;;
    *) return 1 ;;
  esac
}

retire_owner() {
  if [ ! -f "$owner_file" ]; then
    sleep 0.02
    rmdir "$lifecycle_dir" 2>/dev/null || true
    return
  fi
  previous_pid=$(owner_field 1)
  previous_child=$(owner_field 2)
  if valid_pid "$previous_pid" &&
    process_matches "$previous_pid" "$launcher_path"; then
    stop_pid "$previous_pid"
  fi
  if [ "$previous_child" != "$previous_pid" ] &&
    valid_pid "$previous_child" &&
    process_matches "$previous_child" "$binary_path"; then
    stop_pid "$previous_child"
  fi
  rm -f "$owner_file"
  rmdir "$lifecycle_dir" 2>/dev/null || true
}

claim_lifecycle() {
  attempts=0
  while ! mkdir "$lifecycle_dir" 2>/dev/null; do
    retire_owner
    attempts=$((attempts + 1))
    if [ "$attempts" -ge 100 ]; then
      echo "Cannot claim web debug lifecycle: $lifecycle_dir" >&2
      exit 1
    fi
  done
  write_owner
}

write_owner() {
  temporary_owner="$owner_file.$$"
  printf '%s %s\n' "$$" "$child_pid" > "$temporary_owner"
  mv "$temporary_owner" "$owner_file"
}

cleanup() {
  status=$?
  trap - EXIT HUP INT TERM
  if valid_pid "$child_pid" && kill -0 "$child_pid" 2>/dev/null; then
    kill -TERM "$child_pid" 2>/dev/null || true
    wait "$child_pid" 2>/dev/null || true
  fi
  test -z "$run_cache" || rm -rf "$run_cache"
  if [ -f "$owner_file" ] && [ "$(owner_field 1)" = "$$" ]; then
    rm -f "$owner_file"
    rmdir "$lifecycle_dir" 2>/dev/null || true
  fi
  exit "$status"
}

terminate() {
  exit 143
}

case "$cache_root" in
  /*/video-debug-cache*) ;;
  *) echo "Unsafe web debug cache root: $cache_root" >&2; exit 1 ;;
esac
case "$lifecycle_dir" in
  /|'') echo "Unsafe web debug lifecycle directory" >&2; exit 1 ;;
  /*) ;;
  *) echo "Web debug lifecycle directory must be absolute" >&2; exit 1 ;;
esac
test -d "$rust_dir" || {
  echo "Rust directory does not exist: $rust_dir" >&2
  exit 1
}

trap cleanup EXIT
trap terminate HUP INT TERM
mkdir -p "$(dirname "$lifecycle_dir")" "$(dirname "$cache_root")"
claim_lifecycle
rm -rf "$cache_root" "$cache_root".*
run_cache=$(mktemp -d "$cache_root.XXXXXX")

cd "$rust_dir"
cargo build --features video-debug-web --bin video-debug
GHOSTR_VIDEO_DEBUG_CACHE="$run_cache" "$binary_path" &
child_pid=$!
write_owner
wait "$child_pid"
child_pid=
