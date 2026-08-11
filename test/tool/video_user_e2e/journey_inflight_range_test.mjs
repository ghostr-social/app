import assert from "node:assert/strict";
import test from "node:test";
import {validateJourney} from "../../../tool/video_user_e2e/journey_outcome.mjs";

test("accepts a truthful range still streaming when smooth evidence is captured", () => {
  const trace = {
    clicks: [{id: "a", at_ms: 0}],
    samples: [sample(100, 0.1, 100_000), sample(800, 0.8, 200_000)],
    requests: [{
      url: "http://127.0.0.1/video.mp4",
      method: "GET",
      range: "bytes=0-",
      status: 206,
      content_range: "bytes 0-3999999/4000000",
    }],
  };

  assert.doesNotThrow(() => validateJourney(trace));
});

function sample(at_ms, current_time, bytes) {
  return {
    at_ms,
    player: {id: "a", phase: "playing", current_time},
    state: {videos: [
      {id: "a", downloaded_bytes: bytes, total_bytes: 4_000_000},
      {id: "b", downloaded_bytes: 100_000, total_bytes: 4_000_000},
    ]},
  };
}
