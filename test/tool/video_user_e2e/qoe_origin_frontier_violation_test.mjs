import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe, requireQoeTargets} from "../../../tool/video_user_e2e/qoe_metrics.mjs";
import {QOE_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

const KIB = 1_024;
const IDS = ["id0", "id1", "id2", "id3", "id4"];

test("origin bytes expose far work hidden inside one ready UI sample", () => {
  const trace = originTrace([
    request("v4", 0),
    request("v0", 1),
    request("v1", 2),
    request("v2", 3),
    request("v3", 4),
  ]);

  const metrics = measureQoe(trace);
  const otherwiseSmooth = {
    ...metrics,
    startup_latency_ms: 0,
    focus_switch_latency_ms: 0,
    rebuffer_ratio: 0,
    cancellation_waste_bytes: 0,
    ahead_prefetch_bytes: 64 * KIB,
  };

  assert.equal(metrics.far_ahead_before_frontier_bytes, 64 * KIB);
  assert.throws(() => requireQoeTargets(otherwiseSmooth, QOE_TARGETS), /far-ahead/);
});

function originTrace(origin_requests) {
  const ready = Object.fromEntries(IDS.map((id) => [id, 64 * KIB]));
  return {
    ordered_video_ids: IDS,
    video_ids: Object.fromEntries(IDS.map((id, index) => [`v${index}`, id])),
    warm_prefetch: {
      focus_started_at_epoch_ms: 1_000,
      baseline_bytes: Object.fromEntries(IDS.map((id) => [id, 0])),
      samples: [{downloaded_bytes: ready}],
      protected_count: 4,
      minimum_bytes: 48 * KIB,
      latency_ms: 10,
    },
    origin_requests,
    clicks: [],
    samples: [],
  };
}

function request(video, ordinal) {
  return {
    video,
    method: "GET",
    started_at_ms: 1_100 + ordinal * 2,
    start_ordinal: ordinal * 2,
    completed: true,
    chunk_events: [{at_ms: 1_101 + ordinal * 2, ordinal: ordinal * 2 + 1,
      bytes: 64 * KIB}],
  };
}
