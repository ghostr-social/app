import assert from "node:assert/strict";
import test from "node:test";
import {poll} from "../../../tool/video_user_e2e/wait.mjs";

test("poll samples readiness once at the end of its final interval", async () => {
  let reads = 0;

  const result = await poll({
    read: async () => ++reads === 1 ? "waiting" : "ready",
    accept: (value) => value === "ready",
    timeoutMs: 5,
    intervalMs: 20,
    label: "final interval",
  });

  assert.equal(result, "ready");
  assert.equal(reads, 2);
});
