import assert from "node:assert/strict";
import test from "node:test";
import {delay} from "../../../tool/video_user_e2e/wait.mjs";

test("an abort rejects an active delay with its exact reason", async () => {
  const controller = new AbortController();
  const reason = new Error("stop waiting");
  const pending = delay(10_000, controller.signal);

  controller.abort(reason);

  await assert.rejects(pending, (error) => error === reason);
});
