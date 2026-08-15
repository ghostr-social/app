import {impairmentVideoRegistration} from "./impairment_plan.mjs";
import {playableMedia} from "./media_fixture.mjs";

const VIDEO_COUNT = 8;

export async function registerOrderedVideos(input) {
  const ids = [];
  for (let index = 0; index < VIDEO_COUNT; index += 1) {
    ids.push(await registerVideo(input, `v${index}`));
  }
  return ids;
}

export async function selectVideoFocus(server, id, request = fetch) {
  const response = await request(`${server}/api/focus`, {
    method: "PUT",
    headers: {"content-type": "application/json"},
    body: JSON.stringify({id}),
  });
  if (!response.ok) throw new Error(`video focus failed: ${response.status}`);
}

async function registerVideo(input, name) {
  const source = impairmentVideoRegistration(input.scenario, name, input.origin);
  const response = await (input.request ?? fetch)(`${input.server}/api/videos`, {
    method: "POST",
    headers: {"content-type": "application/json"},
    body: JSON.stringify({
      ...source,
      size_bytes: input.sizeBytes ?? playableMedia.bytes.length,
      duration_ms: playableMedia.durationMs,
    }),
  });
  if (response.status !== 201) throw new Error(`video registration failed: ${response.status}`);
  return (await response.json()).id;
}
