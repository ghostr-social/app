import assert from "node:assert/strict";
import test from "node:test";
import {startLocalOrigin} from "../../../tool/video_user_e2e/local_origin.mjs";
import {delay} from "../../../tool/video_user_e2e/wait.mjs";

test("a canceled range releases its origin task and leaves the fixture usable", async () => {
  const origin = await startLocalOrigin({
    virtualBytes: 100,
    chunkBytes: 1,
    chunkDelayMs: 10,
  });
  try {
    const canceled = await fetch(`${origin.url}/cancel.mp4`, {
      headers: {range: "bytes=0-99"},
    });
    await canceled.body.cancel();
    await delay(50);

    assert.equal(origin.activeRequests(), 0);
    const healthy = await fetch(`${origin.url}/healthy.mp4`, {
      headers: {range: "bytes=0-1"},
    });
    assert.equal((await healthy.arrayBuffer()).byteLength, 2);
  } finally {
    await origin.close();
  }
});
