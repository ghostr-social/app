import assert from "node:assert/strict";
import test from "node:test";
import {startLocalOrigin} from "../../../tool/video_user_e2e/local_origin.mjs";

test("client cancellation before an injection threshold is cancellation", async () => {
  const origin = await startLocalOrigin({
    virtualBytes: 100,
    chunkBytes: 4,
    chunkDelayMs: 50,
    abortEveryNthRequest: 1,
    abortAfterBytes: 12,
  });
  try {
    const response = await fetch(`${origin.url}/planned.mp4`, {
      headers: {range: "bytes=0-99"},
    });
    await response.body.cancel();
    await origin.waitForIdle();

    assert.equal(origin.requests[0].injected_failure, false);
    assert.equal(origin.requests[0].canceled, true);
    assert.ok(origin.requests[0].bytes_sent < 12);
  } finally {
    await origin.close();
  }
});
