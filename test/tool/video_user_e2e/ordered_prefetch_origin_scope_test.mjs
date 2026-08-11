import assert from "node:assert/strict";
import test from "node:test";
import {
  measureOrderedPrefetch, requireOrderedPrefetchTargets,
} from "../../../tool/video_user_e2e/ordered_prefetch_acceptance.mjs";
import {ORDERED_PREFETCH_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

const IDS = Array.from({length: 8}, (_, index) => `id${index}`);

test("far origin use before focus or after readiness fails ordered prefetch", () => {
  const trace = {
    ordered_video_ids: IDS,
    video_ids: Object.fromEntries(IDS.map((id, index) => [`v${index}`, id])),
    warm_prefetch: {focus_started_at_epoch_ms: 1_000, protected_count: 4,
      minimum_bytes: 49_152, latency_ms: 2_000,
      ready_bytes: Object.fromEntries(IDS.slice(0, 4).map((id) => [id, 49_152]))},
    origin_requests: [
      {video: "v4", started_at_ms: 900, start_ordinal: 0,
        chunk_events: [{at_ms: 901, ordinal: 1, bytes: 65_536}]},
      {video: "v4", started_at_ms: 3_100, start_ordinal: 2,
        chunk_events: [{at_ms: 3_101, ordinal: 3, bytes: 65_536}]},
    ],
  };

  const metrics = measureOrderedPrefetch(trace);

  assert.equal(metrics.far_origin_body_bytes, 131_072);
  assert.equal(metrics.far_origin_request_starts, 2);
  assert.throws(
    () => requireOrderedPrefetchTargets(metrics, ORDERED_PREFETCH_TARGETS),
    /far origin body bytes/,
  );
  assert.throws(
    () => requireOrderedPrefetchTargets({...metrics, far_origin_body_bytes: 0},
      ORDERED_PREFETCH_TARGETS),
    /far origin request starts/,
  );
});
