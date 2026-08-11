import assert from "node:assert/strict";
import test from "node:test";
import {validateJourney} from "../../../tool/video_user_e2e/journey_outcome.mjs";

test("accepts smooth playback, prefetch before EOF, and a fast visible jump", () => {
  const trace = {
    clicks: [{id: "b", at_ms: 1_000}],
    samples: [
      sample({id: "a", at: 100, time: 0.2, currentBytes: 300, aheadBytes: 280}),
      sample({id: "a", at: 700, time: 0.9, currentBytes: 500, aheadBytes: 420}),
      sample({id: "b", at: 1_700, time: 0.5, currentBytes: 900, aheadBytes: 700}),
      sample({id: "b", at: 2_300, time: 1.2, currentBytes: 1_100, aheadBytes: 700}),
    ],
    requests: [
      {url: "http://127.0.0.1/debug/video_form.js", status: 200},
      {url: "data:image/svg+xml;base64,PHN2Zy8+", status: 200, finished: true},
      {url: "http://127.0.0.1/video.mp4", range: "bytes=0-255",
        status: 206, content_range: "bytes 0-255/4000", finished: true},
    ],
  };

  assert.doesNotThrow(() => validateJourney(trace, {maxJumpMs: 2_000}));
});

test("rejects a journey whose media time never advances", () => {
  const trace = {
    clicks: [],
    samples: [
      sample({id: "a", at: 100, time: 0, currentBytes: 300, aheadBytes: 0}),
      sample({id: "a", at: 700, time: 0, currentBytes: 500, aheadBytes: 0}),
    ],
    requests: [{url: "http://127.0.0.1/video.mp4", range: "bytes=0-255",
      status: 206, content_range: "bytes 0-255/4000", finished: true}],
  };

  assert.throws(() => validateJourney(trace), /media time did not advance/);
});

function sample(input) {
  return {
    at_ms: input.at,
    player: {id: input.id, current_time: input.time, phase: "playing"},
    state: {
      videos: [
        {id: input.id, downloaded_bytes: input.currentBytes, total_bytes: 4_000},
        {id: input.id === "a" ? "b" : "a", downloaded_bytes: input.aheadBytes,
          total_bytes: 4_000},
      ],
    },
  };
}
