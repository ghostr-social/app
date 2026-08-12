import assert from "node:assert/strict";
import test from "node:test";
import {startLocalOrigin} from "../../../tool/video_user_e2e/local_origin.mjs";

test("packet loss aborts a deterministic request after a deterministic byte count", async () => {
  const origin = await startLocalOrigin({
    virtualBytes: 100,
    chunkBytes: 4,
    abortEveryNthRequest: 2,
    abortAfterBytes: 12,
  });
  try {
    const first = await fetch(`${origin.url}/first.mp4`, {headers: {range: "bytes=0-19"}});
    assert.equal((await first.arrayBuffer()).byteLength, 20);

    const lost = await fetch(`${origin.url}/lost.mp4`, {headers: {range: "bytes=0-19"}});
    await assert.rejects(() => lost.arrayBuffer());
    assert.deepEqual(origin.requests.map((entry) => ({
      id: entry.id,
      bytes_sent: entry.bytes_sent,
      completed: entry.completed,
      injected_failure: entry.injected_failure,
    })), [
      {id: "first", bytes_sent: 20, completed: true, injected_failure: false},
      {id: "lost", bytes_sent: 12, completed: false, injected_failure: true},
    ]);
  } finally {
    await origin.close();
  }
});
