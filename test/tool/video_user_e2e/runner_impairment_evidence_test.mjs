import assert from "node:assert/strict";
import test from "node:test";
import {createVideoUserE2eRunner} from "../../../tool/video_user_e2e/runner.mjs";
import {successfulRunnerBoundaries} from "./runner_test_support.mjs";

test("the runner retains successful bootstrap impairment application", async () => {
  const fixture = successfulRunnerBoundaries();
  const run = createVideoUserE2eRunner(fixture.boundaries);

  const result = await run({
    root: "/tmp/video-runner",
    environment: {},
    browser: {},
    scenario: "high_rtt",
  });

  assert.equal(result.trace.impairments.length, 1);
  assert.deepEqual(result.trace.impairments[0].payload, {
    bandwidth_kbps: 2_500,
    latency_ms: 450,
    max_connections_per_host: 3,
  });
  assert.ok(Number.isFinite(result.trace.impairments[0].applied_at_epoch_ms));
});
