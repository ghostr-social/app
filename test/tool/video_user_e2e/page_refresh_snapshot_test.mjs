import assert from "node:assert/strict";
import test from "node:test";
import {refreshDebugSnapshot} from "../../../tool/video_user_e2e/page_runtime.mjs";

test("a harness refresh returns newly rendered debug state", async () => {
  const calls = [];
  const page = {
    sessionId: "page",
    cdp: {send: async (method, params, sessionId) => {
      calls.push({method, params, sessionId});
      return {result: {value: {videos: [{id: "v0"}]}}};
    }},
  };

  const state = await refreshDebugSnapshot(page);

  assert.deepEqual(state, {videos: [{id: "v0"}]});
  assert.equal(calls[0].method, "Runtime.evaluate");
  assert.match(calls[0].params.expression, /refresh\(\).*latestState/);
  assert.equal(calls[0].sessionId, "page");
});
