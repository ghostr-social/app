import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe, requireQoeTargets} from "../../../tool/video_user_e2e/qoe_metrics.mjs";
import {QOE_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

const KIB = 1_024;
const FLOOR = 48 * KIB;
const IDS = Array.from({length: 8}, (_, index) => `id${index}`);

test("a second focus resets the protected frontier before admitting farther work", () => {
  const trace = readyInitialTrace();
  trace.focus_locality_epochs = [{
    focus_id: IDS[1],
    started_at_epoch_ms: 2_000,
    ended_at_epoch_ms: 3_000,
    started_after_origin_ordinal: 3,
    ended_through_origin_ordinal: 5,
    protected_ids: IDS.slice(1, 5),
    baseline_bytes: bytes(0),
    minimum_bytes: FLOOR,
  }];
  trace.origin_requests = [{
    video: "v5",
    started_at_ms: 2_100,
    start_ordinal: 4,
    chunk_events: [{at_ms: 2_101, ordinal: 5, bytes: 64 * KIB}],
  }];

  const metrics = measureQoe(trace);

  assert.equal(metrics.far_ahead_before_frontier_bytes, 64 * KIB);
  assert.equal(metrics.far_ahead_request_starts_before_frontier, 1);
  assert.throws(() => requireQoeTargets(smooth(metrics), QOE_TARGETS), /far-ahead/);
});

function readyInitialTrace() {
  return {
    ordered_video_ids: IDS,
    video_ids: Object.fromEntries(IDS.map((id, index) => [`v${index}`, id])),
    warm_prefetch: {
      focus_started_at_epoch_ms: 1_000,
      baseline_bytes: bytes(FLOOR),
      samples: [],
      protected_count: 4,
      minimum_bytes: FLOOR,
      latency_ms: 10,
    },
    clicks: [],
    samples: [],
  };
}

function bytes(value) {
  return Object.fromEntries(IDS.map((id) => [id, value]));
}

function smooth(metrics) {
  return {...metrics, startup_latency_ms: 0, focus_switch_latency_ms: 0,
    rebuffer_ratio: 0, cancellation_waste_bytes: 0, ahead_prefetch_bytes: FLOOR,
    duplicate_completed_origin_bytes: 0, protected_transition_latency_ms: 0};
}
