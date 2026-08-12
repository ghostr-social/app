import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe} from "../../../tool/video_user_e2e/qoe_metrics.mjs";

test("cancellation waste counts only chunks sent after focus left that video", () => {
  const trace = {
    started_at_epoch_ms: 10_000,
    video_ids: {a: "post-a", b: "post-b"},
    clicks: [{id: "post-a", at_ms: 0}, {id: "post-b", at_ms: 1_000}],
    samples: [
      sample("post-a", 100, "a", "b"),
      sample("post-b", 1_100, "b", "a"),
      sample("post-b", 2_100, "b", "a"),
    ],
    origin_requests: [{
      video: "a",
      canceled: true,
      completed: false,
      bytes_sent: 196_608,
      chunk_events: [
        {at_ms: 10_500, bytes: 65_536},
        {at_ms: 11_050, bytes: 65_536},
        {at_ms: 11_100, bytes: 65_536},
      ],
    }],
  };

  assert.equal(measureQoe(trace).cancellation_waste_bytes, 131_072);
});

function sample(id, at_ms, current, ahead) {
  return {
    at_ms,
    player: {id, phase: "playing", current_time: at_ms / 1_000},
    state: {videos: [
      {id, downloaded_bytes: 100_000, total_bytes: 4_000_000},
      {id: id === "post-a" ? "post-b" : "post-a", downloaded_bytes: 65_536,
        total_bytes: 4_000_000},
    ]},
    labels: {current, ahead},
  };
}
