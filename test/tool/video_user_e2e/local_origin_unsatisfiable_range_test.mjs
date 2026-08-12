import assert from "node:assert/strict";
import test from "node:test";
import {startLocalOrigin} from "../../../tool/video_user_e2e/local_origin.mjs";

test("the local origin truthfully rejects an unsatisfiable media range", async () => {
  const origin = await startLocalOrigin({virtualBytes: 100});
  try {
    const response = await fetch(`${origin.url}/video.mp4`, {
      headers: {range: "bytes=100-101"},
    });

    assert.equal(response.status, 416);
    assert.equal(response.headers.get("content-range"), "bytes */100");
    assert.equal((await response.arrayBuffer()).byteLength, 0);
    assert.deepEqual(origin.requests, []);
  } finally {
    await origin.close();
  }
});
