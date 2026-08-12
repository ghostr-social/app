import assert from "node:assert/strict";
import test from "node:test";
import {createVideoUserE2eRunner} from "../../../tool/video_user_e2e/runner.mjs";
import {successfulRunnerBoundaries} from "./runner_test_support.mjs";

test("a moving journey carries adaptive plans without a fixed locality window", async () => {
  const fixture = successfulRunnerBoundaries();
  const run = createVideoUserE2eRunner(fixture.boundaries);

  const result = await run({root: "/tmp/video-runner", environment: {}, browser: {}});

  assert.equal(result.trace.clicks.length, 4);
  assert.equal(Object.hasOwn(result.trace, "warm_prefetch"), false);
  assert.equal(result.trace.qoe.plan_revision_count, 1);
  assert.equal(Object.hasOwn(result.trace, "focus_locality_epochs"), false);
  assert.equal(fixture.trace(), result.trace);
  assert.ok(fixture.events.includes("focus:video-0"));
  assert.deepEqual(fixture.events.slice(-3), [
    "browser.close", "origin.close", "files.remove",
  ]);
});
