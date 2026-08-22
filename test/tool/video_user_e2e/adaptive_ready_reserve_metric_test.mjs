import assert from "node:assert/strict";
import test from "node:test";
import {
  measureAdaptivePlans, requireAdaptivePlanEvidence,
} from "../../../tool/video_user_e2e/adaptive_plan_acceptance.mjs";
import {
  adaptiveTrace, allocation, plan, readyReserve,
} from "./adaptive_plan_test_support.mjs";

test("adaptive evidence measures the rolling ready reserve", () => {
  const trace = adaptiveTrace({adaptive_plans: [plan({
    mode: "safety",
    ready_reserve: readyReserve({target: 3, ready: 1, structural: 1, protected: 3,
      recovery_horizon_ms: 2_400, underflow_risk_bps: 320,
      ready_coverage_ms: 5_800, candidates: [
        {post_id: "post-1", status: "ready"},
        {post_id: "post-2", status: "structural"},
        {post_id: "post-3", status: "planned"},
      ]}),
    allocations: [allocation({reason: "next_startability"})],
  })]});

  assert.doesNotThrow(() => requireAdaptivePlanEvidence(trace));
  assert.deepEqual(measureAdaptivePlans(trace).ready_reserve, {
    maximum_target: 3,
    maximum_ready: 1,
    maximum_structural: 1,
    maximum_protected: 3,
    maximum_coverage_ms: 5_800,
  });
});
