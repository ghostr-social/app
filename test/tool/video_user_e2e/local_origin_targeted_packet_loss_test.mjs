import assert from "node:assert/strict";
import test from "node:test";
import {startLocalOrigin} from "../../../tool/video_user_e2e/local_origin.mjs";

test("targeted packet loss fails two v2 bodies and no other video", async () => {
  const origin = await startLocalOrigin({
    virtualBytes: 100,
    chunkBytes: 4,
    chunkDelayMs: 0,
    abortFirstAttempts: {video: "v2", count: 2},
    abortAfterBytes: 12,
  });
  try {
    assert.equal((await fetch(`${origin.url}/v2.mp4`, {method: "HEAD"})).status, 200);
    assert.equal(await bodyLength(origin.url, "v0"), 20);
    assert.equal(await bodyLength(origin.url, "v2", "bytes=0-7"), 8);
    await assert.rejects(() => bodyLength(origin.url, "v2"));
    assert.equal(await bodyLength(origin.url, "v1"), 20);
    await assert.rejects(() => bodyLength(origin.url, "v2"));
    assert.equal(await bodyLength(origin.url, "v2"), 20);

    assert.deepEqual(origin.requests.map(result), [
      ["v2", 0, false, true],
      ["v0", 20, false, true],
      ["v2", 8, false, true],
      ["v2", 12, true, false],
      ["v1", 20, false, true],
      ["v2", 12, true, false],
      ["v2", 20, false, true],
    ]);
  } finally {
    await origin.close();
  }
});

async function bodyLength(origin, video, range = "bytes=0-19") {
  const response = await fetch(`${origin}/${video}.mp4`, {
    headers: {range},
  });
  return (await response.arrayBuffer()).byteLength;
}

function result(request) {
  return [request.video, request.bytes_sent, request.injected_failure, request.completed];
}
