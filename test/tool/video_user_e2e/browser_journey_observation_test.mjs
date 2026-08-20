import assert from "node:assert/strict";
import test from "node:test";
import {watchProgress} from "../../../tool/video_user_e2e/browser_journey.mjs";

test("the browser journey observes the requested media-time window", async () => {
  const times = [0, 1, 2.5];
  const trace = {samples: []};
  const page = pageReturning(times);

  await watchProgress({
    page,
    id: "protected-video",
    trace,
    started: Date.now(),
    observedSeconds: 2.5,
  });

  assert.deepEqual(trace.samples.map((sample) => sample.player.current_time), times);
});

function pageReturning(times) {
  let index = 0;
  return {sessionId: "page", cdp: {send: async (_, input) => {
    if (input.expression.includes("document.getElementById")) {
      const current_time = times[Math.min(index++, times.length - 1)];
      return {result: {value: player(current_time)}};
    }
    return {result: {value: {videos: []}}};
  }}};
}

function player(current_time) {
  return {id: "protected-video", phase: "playing", presented: true,
    current_time, paused: false};
}
