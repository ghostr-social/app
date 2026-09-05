#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
serial=${1:-}
test -n "$serial" || { echo 'Physical Android serial is required.' >&2; exit 64; }
stamp=$(date -u +%Y%m%dT%H%M%SZ)
export LIVE_VIDEO_EVIDENCE_DIR="$root/.artifacts/live-video/$stamp"
mkdir -p "$LIVE_VIDEO_EVIDENCE_DIR"
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}
status=0
sh "$root/tool/run_warp_evidence.sh" "$serial" live-nostr \
  sh "$root/tool/run_video_live_android.sh" "$serial" \
  >"$LIVE_VIDEO_EVIDENCE_DIR/runner.log" 2>&1 || status=$?
cat "$LIVE_VIDEO_EVIDENCE_DIR/runner.log"
dir=$(sed -n 's/^WARP_EVIDENCE_DIR=//p' "$LIVE_VIDEO_EVIDENCE_DIR/runner.log" | tail -1)
if test -n "$dir"; then
  for artifact in report.json final.png; do
    if test -f "$LIVE_VIDEO_EVIDENCE_DIR/$artifact"; then
      gzip -c "$LIVE_VIDEO_EVIDENCE_DIR/$artifact" >"$dir/$artifact.gz"
    fi
  done
fi
exit "$status"
