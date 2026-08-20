import assert from "node:assert/strict";
import test from "node:test";
import {validateJourney} from "../../../tool/video_user_e2e/journey_outcome.mjs";

test("does not classify focus startup buffering as a playback stall", () => {
  const trace = {
    clicks: [
      {id: "a", at_ms: 0},
      {id: "b", at_ms: 1_000},
      {id: "a", at_ms: 2_000},
      {id: "b", at_ms: 3_000},
    ],
    samples: [
      sample("a", "loading", 0, 0),
      sample("a", "playing", 100, 0.1),
      sample("a", "playing", 900, 0.9),
      sample("b", "buffering", 1_000, 0),
      sample("b", "playing", 1_100, 0.1),
      sample("b", "playing", 1_900, 0.9),
      sample("a", "buffering", 2_000, 0),
      sample("a", "playing", 2_100, 0.1),
      sample("a", "playing", 2_900, 0.9),
      sample("b", "buffering", 3_000, 0),
      sample("b", "playing", 3_100, 0.1),
      sample("b", "playing", 3_900, 0.9),
    ],
    requests: [{
      url: "http://127.0.0.1/video.mp4",
      range: "bytes=0-255",
      status: 206,
      content_range: "bytes 0-255/4000",
      finished: true,
    }],
  };

  assert.doesNotThrow(() => validateJourney(trace));
});

function sample(id, phase, at_ms, current_time) {
  return {
    at_ms,
    player: {id, phase, presented: phase === "playing", current_time},
    state: {videos: [
      {id, downloaded_bytes: 1_000, total_bytes: 4_000},
      {id: id === "a" ? "b" : "a", downloaded_bytes: 500, total_bytes: 4_000},
    ]},
  };
}
