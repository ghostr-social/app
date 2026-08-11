import assert from "node:assert/strict";
import test from "node:test";
import {poll} from "../../../tool/video_user_e2e/wait.mjs";

test("poll describes an unserializable final sample on timeout", async () => {
  const sample = {};
  sample.self = sample;

  await assert.rejects(poll({
    read: async () => sample,
    accept: () => false,
    timeoutMs: 1,
    intervalMs: 20,
    label: "circular sample",
  }), /circular sample timed out after 1 ms: \[object Object\]/);
});
