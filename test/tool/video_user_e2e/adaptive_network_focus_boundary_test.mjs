import assert from "node:assert/strict";
import test from "node:test";
import {requireAdaptiveScenarioOutcome} from "../../../tool/video_user_e2e/adaptive_scenario_acceptance.mjs";
import {
  adaptiveTrace, allocation, plan,
} from "./adaptive_plan_test_support.mjs";

test("focus-only breadth contraction is not network adaptation evidence", () => {
  const first = allocation({post_id: "post-0"});
  const second = allocation({post_id: "post-1"});
  const trace = adaptiveTrace({
    scenario: "bandwidth_drop",
    started_at_epoch_ms: 0,
    clicks: [{id: "post-1", at_ms: 201}],
    impairments: [receipt(200, 700), receipt(400, 2_500)],
    adaptive_plans: [
      plan({allocations: [first, second]}),
      plan({revision: 2, observed_at_ms: 200, allocations: [first, second]}),
      plan({revision: 3, observed_at_ms: 250, allocations: [second]}),
    ],
  });

  assert.throws(
    () => requireAdaptiveScenarioOutcome(trace),
    /network impairment did not change adaptive allocation evidence/,
  );
});

function receipt(applied_at_epoch_ms, bandwidth_kbps) {
  return {kind: "network", applied_at_epoch_ms, payload: {bandwidth_kbps}};
}
