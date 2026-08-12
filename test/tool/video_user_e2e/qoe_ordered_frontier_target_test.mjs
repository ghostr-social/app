import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe, requireQoeTargets} from "../../../tool/video_user_e2e/qoe_metrics.mjs";
import {QOE_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

const KIB = 1_024;
const FLOOR = 48 * KIB;
const IDS = Array.from({length: 8}, (_, index) => `v${index}`);

test("far-ahead work fails while the ordered protected frontier is underfilled", () => {
  const bad = measureQoe(trace({v0: FLOOR, v6: 64 * KIB}));

  assert.equal(bad.far_ahead_before_frontier_bytes, 65_536);
  assert.throws(
    () => requireQoeTargets(bad, QOE_TARGETS),
    /far-ahead before protected frontier/,
  );

  const ordered = measureQoe(trace({v0: FLOOR, v1: FLOOR, v2: FLOOR, v3: FLOOR}));
  assert.equal(ordered.far_ahead_before_frontier_bytes, 0);
  assert.equal(ordered.ahead_prefetch_bytes, 3 * FLOOR);
  assert.doesNotThrow(() => requireQoeTargets(ordered, QOE_TARGETS));
});

function trace(gains) {
  const baseline = Object.fromEntries(IDS.map((id) => [id, baselineBytes(id)]));
  const downloaded = Object.fromEntries(IDS.map((id) => [id, baseline[id] + (gains[id] ?? 0)]));
  return {
    ordered_video_ids: IDS,
    warm_prefetch: {
      baseline_bytes: baseline,
      samples: [{at_ms: 100, downloaded_bytes: downloaded}],
      protected_count: 4,
      minimum_bytes: FLOOR,
      latency_ms: 1_000,
    },
    clicks: [{id: "v0", at_ms: 0}],
    samples: [player(0), player(100)],
  };
}

function baselineBytes(id) {
  if (id === "v1") return 2 * 1_024 * KIB;
  return id === "v6" ? 8 * KIB : 0;
}

function player(at_ms) {
  return {
    at_ms,
    player: {id: "v0", phase: "playing", current_time: at_ms / 1_000},
    state: {videos: [
      {id: "v0", downloaded_bytes: 128 * KIB, total_bytes: 4_000_000},
      {id: "v1", downloaded_bytes: 2 * 1_024 * KIB + 64 * KIB, total_bytes: 4_000_000},
    ]},
  };
}
