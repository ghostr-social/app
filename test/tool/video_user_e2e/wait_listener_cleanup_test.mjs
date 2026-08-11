import assert from "node:assert/strict";
import test from "node:test";
import {delay} from "../../../tool/video_user_e2e/wait.mjs";

test("completed delays release their abort listener", async () => {
  const listeners = new Set();
  const signal = {
    aborted: false,
    addEventListener: (_, listener) => listeners.add(listener),
    removeEventListener: (_, listener) => listeners.delete(listener),
  };

  await delay(0, signal);

  assert.equal(listeners.size, 0);
});
