import assert from "node:assert/strict";
import test from "node:test";
import {validateJourney} from "../../../tool/video_user_e2e/journey_outcome.mjs";
import {measureQoe} from "../../../tool/video_user_e2e/qoe_metrics.mjs";

test("rapidly superseded focus intents judge only the final visible destination", () => {
  const trace = {
    clicks: [
      {id: "a", at_ms: 0, superseded: true},
      {id: "b", at_ms: 200, superseded: true},
      {id: "a", at_ms: 400, superseded: true},
      {id: "b", at_ms: 600, superseded: false},
    ],
    samples: [sample(700, 0), sample(1_000, 0.2), sample(1_700, 0.9)],
    requests: [{
      url: "http://127.0.0.1/video.mp4",
      method: "GET",
      range: "bytes=0-255",
      status: 206,
      content_range: "bytes 0-255/4000000",
      finished: true,
    }],
  };

  assert.doesNotThrow(() => validateJourney(trace));
  assert.equal(measureQoe(trace).startup_latency_ms, 100);
});

function sample(at_ms, current_time) {
  return {
    at_ms,
    player: {id: "b", phase: "playing", presented: true, current_time},
    state: {videos: [
      {id: "b", downloaded_bytes: 500_000, total_bytes: 4_000_000},
      {id: "a", downloaded_bytes: 200_000, total_bytes: 4_000_000},
    ]},
  };
}
