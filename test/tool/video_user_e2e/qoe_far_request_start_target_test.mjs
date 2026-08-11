import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe, requireQoeTargets} from "../../../tool/video_user_e2e/qoe_metrics.mjs";
import {QOE_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

const KIB = 1_024;
const IDS = ["id0", "id1", "id2", "id3", "id4"];

test("a far request start before the protected frontier fails even with zero far bytes", () => {
  const before = metrics(7);
  const after = metrics(11);

  assert.equal(before.far_ahead_request_starts_before_frontier, 1);
  assert.throws(() => requireQoeTargets(smooth(before), QOE_TARGETS), /far-ahead request starts/);
  assert.equal(after.far_ahead_request_starts_before_frontier, 0);
  assert.doesNotThrow(() => requireQoeTargets(smooth(after), QOE_TARGETS));
});

function metrics(start_ordinal) {
  const chunks = [0, 1, 2, 3].map((index) => ({
    video: `v${index}`,
    method: "GET", started_at_ms: 1_100, start_ordinal: index < 3 ? index * 2 + 1 : 9,
    chunk_events: [{at_ms: 1_101, ordinal: index < 3 ? index * 2 + 2 : 10,
      bytes: 48 * KIB}],
  }));
  return measureQoe(trace([...chunks, {
    video: "v4", method: "GET", started_at_ms: 1_100, start_ordinal, chunk_events: [],
  }]));
}

function trace(origin_requests) {
  return {ordered_video_ids: IDS, video_ids: ids(), origin_requests,
    warm_prefetch: {focus_started_at_epoch_ms: 1_000, baseline_bytes: baseline(),
      samples: [], protected_count: 4, minimum_bytes: 48 * KIB, latency_ms: 10}};
}

function ids() {
  return Object.fromEntries(IDS.map((id, index) => [`v${index}`, id]));
}

function baseline() {
  return Object.fromEntries(IDS.map((id) => [id, 0]));
}

function smooth(metrics_) {
  return {...metrics_, startup_latency_ms: 0, focus_switch_latency_ms: 0,
    rebuffer_ratio: 0, cancellation_waste_bytes: 0, ahead_prefetch_bytes: 48 * KIB};
}
