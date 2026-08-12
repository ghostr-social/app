import assert from "node:assert/strict";
import test from "node:test";
import {watchUntilPlaying} from "../../../tool/video_user_e2e/browser_journey.mjs";

test("the transition watcher returns on the first playing observation", async () => {
  const trace = {samples: []};
  const page = {sessionId: "page", cdp: {send: async (_, input) => {
    const value = input.expression.includes("document.getElementById")
      ? {id: "v0", phase: "playing", current_time: 0}
      : {videos: []};
    return {result: {value}};
  }}};

  await watchUntilPlaying({page, id: "v0", trace, started: Date.now()});

  assert.equal(trace.samples.length, 1);
  assert.equal(trace.samples[0].player.phase, "playing");
});
