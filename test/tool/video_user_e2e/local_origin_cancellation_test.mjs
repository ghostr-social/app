import assert from "node:assert/strict";
import test from "node:test";
import {startLocalOrigin} from "../../../tool/video_user_e2e/local_origin.mjs";

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
    await origin.waitForIdle();

    assert.equal(origin.activeRequests(), 0);
    assert.equal(origin.requests[0].completed, false);
    assert.equal(origin.requests[0].canceled, true);
    assert.ok(origin.requests[0].bytes_sent < 100);
    const healthy = await fetch(`${origin.url}/healthy.mp4`, {
      headers: {range: "bytes=0-1"},
    });
    assert.equal((await healthy.arrayBuffer()).byteLength, 2);
  } finally {
    await origin.close();
  }
});
