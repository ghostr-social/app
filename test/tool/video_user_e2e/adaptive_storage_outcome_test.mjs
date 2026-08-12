import assert from "node:assert/strict";
import test from "node:test";
import {requireAdaptiveScenarioOutcome} from "../../../tool/video_user_e2e/adaptive_scenario_acceptance.mjs";
import {adaptiveTrace, allocation, plan} from "./adaptive_plan_test_support.mjs";

test("storage pressure contracts allocation until the budget expands", () => {
  const trace = adaptiveTrace({
    scenario: "storage_pressure",
    impairments: [storage(40, 50, 2_097_152), storage(250, 300, 67_108_864)],
    adaptive_plans: [
      plan({observed_at_ms: 100, allocations: [], retained: []}),
      plan({revision: 2, observed_at_ms: 275,
        allocations: [allocation({post_id: "post-7"})]}),
    ],
  });

  assert.doesNotThrow(() => requireAdaptiveScenarioOutcome(trace));
});

function storage(requested_at_epoch_ms, applied_at_epoch_ms, budget_bytes) {
  return {kind: "storage", requested_at_epoch_ms, applied_at_epoch_ms,
    payload: {budget_bytes}};
}
