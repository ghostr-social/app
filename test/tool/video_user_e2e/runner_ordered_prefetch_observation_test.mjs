import assert from "node:assert/strict";
import test from "node:test";
import {createVideoUserE2eRunner} from "../../../tool/video_user_e2e/runner.mjs";
import {successfulRunnerBoundaries} from "./runner_test_support.mjs";

test("ordered prefetch observes far origin use shortly after readiness", async () => {
  const fixture = successfulRunnerBoundaries();
  const refresh = fixture.boundaries.refreshDebugSnapshot;
  let scheduled = false;
  let revealFarRequest;
  const farRequest = new Promise((resolve) => { revealFarRequest = resolve; });
  fixture.boundaries.refreshDebugSnapshot = async () => {
    const state = await refresh();
    if (!scheduled && state.videos.every((video) => video.downloaded_bytes >= 49_152)) {
      scheduled = true;
      setTimeout(() => {
        fixture.origin.requests.push(originRequest());
        revealFarRequest();
      }, 0);
    }
    return state;
  };
  fixture.boundaries.delay = async (milliseconds) => {
    fixture.events.push(`observe:${milliseconds}`);
    await farRequest;
  };
  const run = createVideoUserE2eRunner(fixture.boundaries);

  await assert.rejects(run({root: "/tmp/video-runner", environment: {}, browser: {},
    scenario: "ordered_prefetch"}), /far origin body bytes/);
  assert.ok(fixture.events.includes("observe:500"));
});

function originRequest() {
  return {method: "GET", video: "v4", started_at_ms: Date.now(), start_ordinal: 0,
    bytes_sent: 1_024, completed: false,
    chunk_events: [{at_ms: Date.now(), ordinal: 1, bytes: 1_024}]};
}
