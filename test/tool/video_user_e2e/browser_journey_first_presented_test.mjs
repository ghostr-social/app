import assert from "node:assert/strict";
import test from "node:test";
import {watchUntilPresented} from "../../../tool/video_user_e2e/browser_journey.mjs";

test("the startup watcher ignores Playing until a frame is presented", async () => {
  const trace = {samples: []};
  let reads = 0;
  const page = {sessionId: "page", cdp: {send: async (_, input) => {
    const value = input.expression.includes("document.getElementById")
      ? {id: "v0", phase: "playing", presented: reads++ > 0, current_time: 0}
      : {videos: []};
    return {result: {value}};
  }}};

  await watchUntilPresented({page, id: "v0", trace, started: Date.now()});

  assert.equal(trace.samples.length, 2);
  assert.equal(trace.samples[0].player.presented, false);
  assert.equal(trace.samples[1].player.presented, true);
});
