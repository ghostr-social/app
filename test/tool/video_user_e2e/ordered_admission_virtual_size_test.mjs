import assert from "node:assert/strict";
import test from "node:test";
import {registerOrderedVideos} from "../../../tool/video_user_e2e/ordered_admission.mjs";

test("a demo can advertise a larger deterministic virtual media object", async () => {
  const sizes = [];
  const request = async (_url, options) => {
    sizes.push(JSON.parse(options.body).size_bytes);
    return {status: 201, json: async () => ({id: `video-${sizes.length}`})};
  };

  await registerOrderedVideos({
    server: "http://server/debug",
    origin: "http://origin",
    scenario: null,
    sizeBytes: 8 * 1_024 * 1_024,
    request,
  });

  assert.deepEqual(new Set(sizes), new Set([8 * 1_024 * 1_024]));
});
