import assert from "node:assert/strict";
import test from "node:test";
import {playableMedia} from "../../../tool/video_user_e2e/media_fixture.mjs";
import {startLocalOrigin} from "../../../tool/video_user_e2e/local_origin.mjs";

test("the default origin reports the playable fixture's real total", async () => {
  const origin = await startLocalOrigin();
  try {
    const response = await fetch(`${origin.url}/video.mp4`, {
      headers: {range: "bytes=0-0"},
    });

    assert.equal(response.status, 206);
    assert.equal(
      response.headers.get("content-range"),
      `bytes 0-0/${playableMedia.bytes.length}`,
    );
  } finally {
    await origin.close();
  }
});
