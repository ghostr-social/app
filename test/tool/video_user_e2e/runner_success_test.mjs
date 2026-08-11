import assert from "node:assert/strict";
import test from "node:test";
import {createVideoUserE2eRunner} from "../../../tool/video_user_e2e/runner.mjs";
import {successfulRunnerBoundaries} from "./runner_test_support.mjs";

test("a moving journey has cold transitions and explicit initial locality", async () => {
  const fixture = successfulRunnerBoundaries();
  const run = createVideoUserE2eRunner(fixture.boundaries);

  const result = await run({root: "/tmp/video-runner", environment: {}, browser: {}});

  assert.equal(result.trace.clicks.length, 4);
  assert.equal(Object.hasOwn(result.trace, "warm_prefetch"), false);
  assert.equal(result.trace.qoe.far_ahead_before_frontier_bytes, 0);
  assert.equal(result.trace.focus_locality_epochs[0].pre_click, true);
  assert.equal(result.trace.focus_locality_epochs[0].started_after_origin_ordinal, -1);
  assert.equal(fixture.trace(), result.trace);
  assert.ok(fixture.events.includes("focus:video-0"));
  assert.deepEqual(fixture.events.slice(-3), [
    "browser.close", "origin.close", "files.remove",
  ]);
});
