import assert from "node:assert/strict";
import test from "node:test";
import {validateJourney} from "../../../tool/video_user_e2e/journey_outcome.mjs";

test("ahead work completed during warm-up satisfies the journey", () => {
  const trace = {
    warm_prefetch: {
      ordered_ids: ["a", "b"],
      baseline_bytes: {a: 0, b: 0},
      samples: [{downloaded_bytes: {a: 400, b: 300}}],
    },
    clicks: [],
    samples: [
      sample(100, 0.1),
      sample(700, 0.8),
    ],
    requests: [{
      url: "http://127.0.0.1/video.mp4",
      range: "bytes=0-399",
      status: 206,
      content_range: "bytes 0-399/400",
      finished: true,
    }],
  };

  assert.doesNotThrow(() => validateJourney(trace));
});

function sample(at_ms, current_time) {
  return {
    at_ms,
    player: {id: "a", current_time, phase: "playing"},
    state: {videos: [
      {id: "a", downloaded_bytes: 400, total_bytes: 400},
      {id: "b", downloaded_bytes: 300, total_bytes: 400},
    ]},
  };
}
