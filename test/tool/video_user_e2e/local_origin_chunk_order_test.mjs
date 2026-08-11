import assert from "node:assert/strict";
import test from "node:test";
import {startLocalOrigin} from "../../../tool/video_user_e2e/local_origin.mjs";

test("concurrent origin starts and chunks retain one exact global order", async () => {
  const origin = await startLocalOrigin({virtualBytes: 16, chunkBytes: 4, chunkDelayMs: 1});
  try {
    await Promise.all([
      fetch(`${origin.url}/a.mp4`, {headers: {range: "bytes=0-7"}}),
      fetch(`${origin.url}/b.mp4`, {headers: {range: "bytes=8-15"}}),
    ]);
    await origin.waitForIdle();
    const events = origin.requests.flatMap((request) => [
      request.start_ordinal,
      ...request.chunk_events.map((event) => event.ordinal),
    ]);
    const ordinals = events.sort((a, b) => a - b);

    assert.deepEqual(ordinals, [0, 1, 2, 3, 4, 5]);
    assert.ok(origin.requests.every((request) => {
      return request.chunk_events.every((event) => request.start_ordinal < event.ordinal);
    }));
  } finally {
    await origin.close();
  }
});
