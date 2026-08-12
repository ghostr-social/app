import assert from "node:assert/strict";
import test from "node:test";
import {requireAdaptiveScenarioOutcome} from "../../../tool/video_user_e2e/adaptive_scenario_acceptance.mjs";
import {adaptiveTrace, allocation, plan} from "./adaptive_plan_test_support.mjs";

test("rapid swiping produces explicit breadth-allocation evidence", () => {
  const trace = adaptiveTrace({
    scenario: "rapid_swipes",
    adaptive_plans: [plan({allocations: [allocation({
      post_id: "post-1",
      reason: "rapid_navigation_coverage",
      authority: "transition",
    })]})],
  });

  assert.doesNotThrow(() => requireAdaptiveScenarioOutcome(trace));
});
