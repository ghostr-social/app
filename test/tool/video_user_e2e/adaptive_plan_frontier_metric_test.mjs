import assert from "node:assert/strict";
import test from "node:test";
import {measureAdaptivePlans} from "../../../tool/video_user_e2e/adaptive_plan_acceptance.mjs";
import {
  adaptiveTrace, allocation, plan,
} from "./adaptive_plan_test_support.mjs";

test("adaptive metrics preserve observed variable frontier sizes", () => {
  const trace = adaptiveTrace({adaptive_plans: [
    plan({revision: 1, allocations: [allocation()]}),
    plan({revision: 2, observed_at_ms: 200, allocations: [
      allocation(), allocation({post_id: "post-1", reason: "likely_next_transition"}),
    ]}),
  ]});

  assert.deepEqual(measureAdaptivePlans(trace).frontier_sizes, [1, 2]);
});
