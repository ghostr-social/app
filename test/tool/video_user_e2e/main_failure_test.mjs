import assert from "node:assert/strict";
import test from "node:test";
import {reportVideoE2eFailure} from "../../../tool/video_user_e2e/main.mjs";

test("the main adapter reports a concise failure and sets a failing exit code", () => {
  const messages = [];
  const processState = {exitCode: 0};

  reportVideoE2eFailure(new Error("broken"), (message) => messages.push(message), processState);

  assert.deepEqual(messages, ["video-user-e2e: broken"]);
  assert.equal(processState.exitCode, 1);
});
