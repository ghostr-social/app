import assert from "node:assert/strict";
import test from "node:test";
import {startLocalOrigin} from "../../../tool/video_user_e2e/local_origin.mjs";

test("one failed source leaves its mirror healthy", async () => {
  const origin = await startLocalOrigin({virtualBytes: 100, failSource: "primary"});
  try {
    const primary = await fetch(`${origin.url}/a-primary.mp4`, {
      headers: {range: "bytes=0-9"},
    });
    const mirror = await fetch(`${origin.url}/mirror.mp4`, {
      headers: {range: "bytes=0-9"},
    });

    assert.equal(primary.status, 503);
    assert.equal(mirror.status, 206);
    assert.equal((await mirror.arrayBuffer()).byteLength, 10);
  } finally {
    await origin.close();
  }
});
