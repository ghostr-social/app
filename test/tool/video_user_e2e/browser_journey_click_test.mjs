import assert from "node:assert/strict";
import test from "node:test";
import {clickVideo} from "../../../tool/video_user_e2e/browser_journey.mjs";

test("the browser journey dispatches one trusted visible play click", async () => {
  const calls = [];
  const page = {sessionId: "page", cdp: {send: async (method, input) => {
    calls.push({method, input});
    if (method === "Runtime.evaluate") {
      return {result: {value: {ready: true, x: 10, y: 20, label: "Play video"}}};
    }
    return {};
  }}};

  await clickVideo(page, "video");

  assert.deepEqual(calls.slice(1).map((call) => call.input.type), [
    "mouseMoved", "mousePressed", "mouseReleased",
  ]);
});
