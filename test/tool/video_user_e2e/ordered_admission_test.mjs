import assert from "node:assert/strict";
import test from "node:test";
import {
  registerOrderedVideos, selectVideoFocus,
} from "../../../tool/video_user_e2e/ordered_admission.mjs";

test("eight ordered registrations are followed by an explicit first-video focus", async () => {
  const requests = [];
  const request = async (url, options) => {
    requests.push({url, ...options, body: JSON.parse(options.body)});
    const ordinal = requests.length;
    return {ok: true, status: ordinal === 9 ? 204 : 201, json: async () => ({id: `post-v${ordinal - 1}`})};
  };

  const ids = await registerOrderedVideos({
    server: "http://server/debug",
    origin: "http://origin",
    scenario: null,
    request,
  });
  await selectVideoFocus("http://server/debug", ids[0], request);

  assert.deepEqual(ids, Array.from({length: 8}, (_, index) => `post-v${index}`));
  assert.equal(requests[0].url, "http://server/debug/api/videos");
  assert.deepEqual(requests.slice(0, 8).map((entry) => entry.body.url),
    Array.from({length: 8}, (_, index) => `http://origin/v${index}.mp4`));
  assert.deepEqual(requests[8], {
    url: "http://server/debug/api/focus",
    method: "PUT",
    headers: {"content-type": "application/json"},
    body: {id: "post-v0"},
  });
});
