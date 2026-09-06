#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
serial=${1:-}
test -n "$serial" || { echo 'Physical Android serial is required.' >&2; exit 64; }
case "$serial" in emulator-*) echo 'Physical hardware is required.' >&2; exit 64;; esac
adb_bin=${ADB:-adb}
test "$("$adb_bin" -s "$serial" get-state)" = device
qemu=$("$adb_bin" -s "$serial" shell getprop ro.kernel.qemu | tr -d '\r')
test "$qemu" != 1 || { echo 'Physical hardware is required.' >&2; exit 64; }
abi=$("$adb_bin" -s "$serial" shell getprop ro.product.cpu.abi | tr -d '\r')
case "$abi" in
  arm64-v8a) target=android-arm64;;
  armeabi-v7a) target=android-arm;;
  x86_64) target=android-x64;;
  *) echo "Unsupported connected Android ABI: $abi" >&2; exit 64;;
esac
cd "$root"
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export LIVE_VIDEO_EVIDENCE_DIR=${LIVE_VIDEO_EVIDENCE_DIR:-$root/.artifacts/live-video}
mkdir -p "$LIVE_VIDEO_EVIDENCE_DIR"
installed_path=$("$adb_bin" -s "$serial" shell pm path app.ghostr | tr -d '\r' | sed -n 's/^package:\(.*\/base.apk\)$/\1/p')
test -n "$installed_path" || { echo 'The signed-in app must already be installed.' >&2; exit 64; }
"$adb_bin" -s "$serial" pull "$installed_path" "$LIVE_VIDEO_EVIDENCE_DIR/restore.apk" >/dev/null
installed=0
port=
restore_app() {
  if test "$installed" = 1; then
    "$adb_bin" -s "$serial" shell am force-stop app.ghostr || true
    "$adb_bin" -s "$serial" install -r "$LIVE_VIDEO_EVIDENCE_DIR/restore.apk" || return
    "$adb_bin" -s "$serial" shell am start -n app.ghostr/social.ghostr.MainActivity || true
  fi
  if test -n "$port"; then "$adb_bin" -s "$serial" forward --remove "tcp:$port" || true; fi
  rm -f "$LIVE_VIDEO_EVIDENCE_DIR/restore.apk"
}
trap restore_app EXIT
flutter_bin=${FLUTTER:-flutter}
python3 tool/live_video_prior_corpus.py "$root/evidence/warp" \
  >"$LIVE_VIDEO_EVIDENCE_DIR/corpus-defines.json"
"$flutter_bin" build apk --profile --no-pub --target-platform "$target" \
  --target integration_test/live_nostr_video_test.dart \
  --dart-define-from-file="$LIVE_VIDEO_EVIDENCE_DIR/corpus-defines.json" \
  --dart-define="LIVE_VIDEO_COUNT=${LIVE_VIDEO_COUNT:-20}" \
  --dart-define="LIVE_COLD_CACHE=${LIVE_COLD_CACHE:-false}" \
  --dart-define="LIVE_COLD_CACHE_KEY=${LIVE_COLD_CACHE_KEY:-}" \
  --dart-define="LIVE_VIDEO_EVENT_IDS=${LIVE_VIDEO_EVENT_IDS:-}"
# Abort on an incompatible install. The Flutter install fallback can erase data.
"$adb_bin" -s "$serial" install -r build/app/outputs/flutter-apk/app-profile.apk
installed=1
"$adb_bin" -s "$serial" shell am force-stop app.ghostr
"$adb_bin" -s "$serial" shell am start -n app.ghostr/social.ghostr.MainActivity \
  --ez enable-dart-profiling true --ez start-paused true
service=$(python3 tool/live_video_vm_service.py "$adb_bin" "$serial")
port=$(printf '%s' "$service" | cut -d / -f 3 | cut -d : -f 2)
status=0
"$flutter_bin" drive --profile --no-pub --keep-app-running \
  --use-existing-app="$service" --driver test_driver/live_video.dart \
  -d "$serial" || status=$?
"$adb_bin" -s "$serial" exec-out screencap -p >"$LIVE_VIDEO_EVIDENCE_DIR/final.png" || true
exit "$status"
