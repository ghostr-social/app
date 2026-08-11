import assert from "node:assert/strict";
import test from "node:test";
import {validateJourney} from "../../../tool/video_user_e2e/journey_outcome.mjs";

test("the journey rejects a visible focus switch after its deadline", () => {
  const trace = {
    clicks: [{id: "a", at_ms: 0}],
    requests: [{url: "http://127.0.0.1/video.mp4", method: "GET",
      range: "bytes=0-255", status: 206,
      content_range: "bytes 0-255/4000", finished: true}],
    samples: [sample(3_000, 0), sample(3_400, 1)],
  };

  assert.throws(() => validateJourney(trace), /did not play within 2500 ms/);
});

function sample(at_ms, current_time) {
  return {at_ms, player: {id: "a", phase: "playing", current_time},
    state: {videos: [
      {id: "a", downloaded_bytes: 500, total_bytes: 4_000},
      {id: "b", downloaded_bytes: 200, total_bytes: 4_000},
    ]}};
}
