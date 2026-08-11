import assert from "node:assert/strict";
import test from "node:test";
import {startLocalOrigin} from "../../../tool/video_user_e2e/local_origin.mjs";

test("local media origin serves truthful paced byte ranges", async () => {
  const origin = await startLocalOrigin({
    virtualBytes: 1_024, chunkBytes: 4, chunkDelayMs: 1,
  });
  try {
    const response = await fetch(`${origin.url}/a.mp4`, {
      headers: {range: "bytes=10-19"},
    });
    const body = Buffer.from(await response.arrayBuffer());

    assert.equal(response.status, 206);
    assert.equal(response.headers.get("content-range"), "bytes 10-19/1024");
    assert.equal(response.headers.get("content-length"), "10");
    assert.equal(body.length, 10);
    assert.deepEqual(origin.requests.map(({id, start, end}) => ({id, start, end})), [
      {id: "a", start: 10, end: 20},
    ]);
  } finally {
    await origin.close();
  }
});
