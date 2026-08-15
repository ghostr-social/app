import assert from "node:assert/strict";
import test from "node:test";
import {createVideoDemoRunner} from "../../../tool/video_user_e2e/demo_session.mjs";

test("demo starts eight videos and holds the observable session until stopped", async () => {
  const events = [];
  const ids = Array.from({length: 8}, (_, index) => `v${index}`);
  const run = createVideoDemoRunner({
    createRunFiles: async () => ({root: "/repo"}),
    createLifecycle: () => ({teardown: async () => events.push("server.close")}),
    startLocalOrigin: async () => ({url: "http://127.0.0.1:4100",
      close: async () => events.push("origin.close")}),
    startServer: async () => ({url: "http://127.0.0.1:4200/debug"}),
    registerOrderedVideos: async () => ids,
    selectVideoFocus: async (_, id) => events.push(`focus:${id}`),
    applyDemoNetwork: async () => events.push("network:4"),
    waitForStop: async () => events.push("visible"),
    removeTransientRunFiles: async () => events.push("files.remove"),
    output: (line) => events.push(line),
  });

  await run({root: "/repo", environment: {}});

  assert.ok(events.some((event) => event.includes("8 videos")));
  assert.ok(events.some((event) => event.includes("127.0.0.1:4200/debug")));
  assert.ok(events.indexOf("network:4") < events.indexOf("focus:v0"));
  assert.deepEqual(events.slice(-4), [
    "visible", "server.close", "origin.close", "files.remove",
  ]);
});
