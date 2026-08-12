import assert from "node:assert/strict";
import test from "node:test";
import {requireUserStartsPlayback} from "../../../tool/video_user_e2e/browser_journey.mjs";

test("the browser journey rejects playback before user intent", async () => {
  const page = {sessionId: "page", cdp: {send: async () => ({
    result: {value: {id: "early", paused: false}},
  })}};

  await assert.rejects(() => requireUserStartsPlayback(page), /started before/);
});
