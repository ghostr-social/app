import assert from "node:assert/strict";
import test from "node:test";
import {measureOrderedPrefetch} from
  "../../../tool/video_user_e2e/ordered_prefetch_acceptance.mjs";

const IDS = Array.from({length: 8}, (_, index) => `id${index}`);

test("a body GET missing its origin start evidence fails closed", () => {
  const malformed = trace({method: "GET", video: "v4", bytes_sent: 64,
    completed: false, chunk_events: []});
  assert.throws(() => measureOrderedPrefetch(malformed), /origin start timestamp/);

  const head = measureOrderedPrefetch(trace({method: "HEAD", video: "v4",
    started_at_ms: 900, start_ordinal: null, bytes_sent: 0,
    completed: true, chunk_events: []}));
  assert.equal(head.far_origin_request_starts, 0);
  assert.equal(head.far_origin_body_bytes, 0);
});

function trace(request) {
  return {
    ordered_video_ids: IDS,
    video_ids: Object.fromEntries(IDS.map((id, index) => [`v${index}`, id])),
    warm_prefetch: {focus_started_at_epoch_ms: 1_000, protected_count: 4,
      minimum_bytes: 49_152, latency_ms: 2_000,
      ready_bytes: Object.fromEntries(IDS.slice(0, 4).map((id) => [id, 49_152]))},
    origin_requests: [request],
  };
}
