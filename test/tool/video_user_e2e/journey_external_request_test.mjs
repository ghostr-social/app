import assert from "node:assert/strict";
import test from "node:test";
import {validateJourney} from "../../../tool/video_user_e2e/journey_outcome.mjs";

test("rejects a supposedly local journey that contacts an external host", () => {
  const trace = {
    clicks: [],
    requests: [
      {url: "https://cdn.example/player.js", method: "GET", status: 200, finished: true},
      {url: "http://127.0.0.1/video.mp4?id=a", method: "GET", range: "bytes=0-255",
        status: 206, content_range: "bytes 0-255/4000", finished: true},
    ],
    samples: [sample(100, 0.2, 300), sample(700, 0.9, 500)],
  };

  assert.throws(() => validateJourney(trace), /journey left loopback/);
});

function sample(at, time, currentBytes) {
  return {
    at_ms: at,
    player: {id: "a", current_time: time, phase: "playing"},
    state: {videos: [
      {id: "a", downloaded_bytes: currentBytes, total_bytes: 4_000},
      {id: "b", downloaded_bytes: 200, total_bytes: 4_000},
    ]},
  };
}
