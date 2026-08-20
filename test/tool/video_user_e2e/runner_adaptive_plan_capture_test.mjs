import assert from "node:assert/strict";
import test from "node:test";
import {createVideoUserE2eRunner} from "../../../tool/video_user_e2e/runner.mjs";
import {plan} from "./adaptive_plan_test_support.mjs";
import {successfulRunnerBoundaries} from "./runner_test_support.mjs";

test("runner persists planner decisions and evaluation in the artifact", async () => {
  const fixture = successfulRunnerBoundaries();
  const refresh = fixture.boundaries.refreshDebugSnapshot;
  fixture.boundaries.refreshDebugSnapshot = async () => ({
    ...await refresh(),
    adaptive_plans: [plan()],
    decisions: {records: [{sequence: 7}]},
    evaluation: {presentation_samples: 3},
  });
  const run = createVideoUserE2eRunner(fixture.boundaries);

  const result = await run({root: "/tmp/video-runner", environment: {}, browser: {}});

  assert.equal(result.trace.adaptive_plans[0].revision, 1);
  assert.equal(result.trace.decisions.records[0].sequence, 7);
  assert.equal(result.trace.evaluation.presentation_samples, 3);
  assert.deepEqual(result.trace.qoe.frontier_sizes, [1]);
});
