import assert from "node:assert/strict";
import test from "node:test";
import {playableMedia} from "../../../tool/video_user_e2e/media_fixture.mjs";
import {registerOrderedVideos} from "../../../tool/video_user_e2e/ordered_admission.mjs";

test("ordered registrations advertise the real playable media", async () => {
  const bodies = [];
  const request = async (_url, options) => {
    bodies.push(JSON.parse(options.body));
    return {status: 201, json: async () => ({id: `video-${bodies.length}`})};
  };

  await registerOrderedVideos({
    server: "http://server/debug",
    origin: "http://origin",
    scenario: null,
    request,
  });

  assert.ok(bodies.length > 1);
  assert.ok(bodies.every((body) => body.size_bytes === playableMedia.bytes.length));
  assert.ok(bodies.every((body) => body.duration_ms === playableMedia.durationMs));
});
