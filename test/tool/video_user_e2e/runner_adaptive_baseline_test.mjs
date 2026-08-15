import assert from "node:assert/strict";
import test from "node:test";
import {createVideoUserE2eRunner} from "../../../tool/video_user_e2e/runner.mjs";
import {allocation, bodyRequest, plan} from "./adaptive_plan_test_support.mjs";
import {successfulRunnerBoundaries} from "./runner_test_support.mjs";

test("adaptive baseline observes a policy-selected variable frontier", async () => {
  const fixture = successfulRunnerBoundaries();
  fixture.origin.requests.push(
    bodyRequest({video: "v0", started_at_ms: 200, closed_at_ms: 500}),
    bodyRequest({
      video: "v1",
      started_at_ms: 250,
      closed_at_ms: 450,
    }),
  );
  const refresh = fixture.boundaries.refreshDebugSnapshot;
  fixture.boundaries.refreshDebugSnapshot = async () => ({
    ...await refresh(),
    adaptive_plans: [plan({allocations: [
      allocation({post_id: fixture.ids[0]}),
      allocation({post_id: fixture.ids[1], reason: "likely_next_transition"}),
    ]})],
  });
  const run = createVideoUserE2eRunner(fixture.boundaries);

  const result = await run({root: "/tmp/video-runner", environment: {}, browser: {},
    scenario: "adaptive_plans"});

  assert.deepEqual(result.trace.clicks, []);
  assert.equal(result.trace.qoe.maximum_frontier_size, 2);
  assert.equal(result.trace.qoe.peak_parallel_origin_videos, 2);
  assert.ok(fixture.events.includes("observe:4000"));
});
