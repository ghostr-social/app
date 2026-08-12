import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe} from "../../../tool/video_user_e2e/qoe_metrics.mjs";

const IDS = ["id0", "id1", "id2", "id3", "id4"];

test("live origin evidence fails closed when post-focus ordinals are malformed", () => {
  assert.throws(() => measureQoe(trace(null)), /origin_requests must be an array/);
  assert.throws(
    () => measureQoe(trace([{video: "v4", started_at_ms: 1_100, chunk_events: []}])),
    /origin start ordinal/,
  );
  assert.throws(
    () => measureQoe(trace([{
      video: "v4", started_at_ms: 1_100, start_ordinal: 0,
      chunk_events: [{at_ms: 1_101, bytes: 64}],
    }])),
    /origin chunk ordinal/,
  );
  assert.throws(
    () => measureQoe(trace([{
      video: "v4", started_at_ms: 1_100, start_ordinal: 0, chunk_events: {},
    }])),
    /origin chunk events must be an array/,
  );
});

function trace(origin_requests) {
  return {
    ordered_video_ids: IDS,
    video_ids: Object.fromEntries(IDS.map((id, index) => [`v${index}`, id])),
    warm_prefetch: {
      focus_started_at_epoch_ms: 1_000,
      baseline_bytes: Object.fromEntries(IDS.map((id) => [id, 0])),
      samples: [],
      protected_count: 4,
      minimum_bytes: 48 * 1_024,
      latency_ms: 10,
    },
    origin_requests,
    clicks: [],
    samples: [],
  };
}
