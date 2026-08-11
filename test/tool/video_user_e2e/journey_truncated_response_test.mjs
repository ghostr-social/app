import assert from "node:assert/strict";
import test from "node:test";
import {validateJourney} from "../../../tool/video_user_e2e/journey_outcome.mjs";

test("rejects a media response whose Range headers precede a body failure", () => {
  const trace = {
    clicks: [],
    requests: [{
      url: "http://127.0.0.1/video.mp4?id=a",
      method: "GET",
      range: "bytes=0-255",
      status: 206,
      content_range: "bytes 0-255/4000",
      failure: "net::ERR_FAILED",
    }],
    samples: [
      sample(100, 0.2, 300, 280),
      sample(700, 0.9, 500, 420),
    ],
  };

  assert.throws(() => validateJourney(trace), /media response did not complete/);
});

function sample(at, time, currentBytes, aheadBytes) {
  return {
    at_ms: at,
    player: {id: "a", current_time: time, phase: "playing"},
    state: {videos: [
      {id: "a", downloaded_bytes: currentBytes, total_bytes: 4_000},
      {id: "b", downloaded_bytes: aheadBytes, total_bytes: 4_000},
    ]},
  };
}
