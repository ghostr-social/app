import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe} from "../../../tool/video_user_e2e/qoe_metrics.mjs";

const KIB = 1_024;
const IDS = ["id0", "id1", "id2", "id3", "id4"];

test("origin bytes allow far work only after the protected frontier", () => {
  const requests = ["v0", "v1", "v2", "v3", "v4"].map(request);
  const ready = Object.fromEntries(IDS.map((id) => [id, 64 * KIB]));
  const trace = {
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
    origin_requests: requests,
    clicks: [],
    samples: [],
  };

  assert.equal(measureQoe(trace).far_ahead_before_frontier_bytes, 0);
});

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
