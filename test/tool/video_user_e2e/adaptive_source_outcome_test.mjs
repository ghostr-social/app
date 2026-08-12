import assert from "node:assert/strict";
import test from "node:test";
import {requireAdaptiveScenarioOutcome} from "../../../tool/video_user_e2e/adaptive_scenario_acceptance.mjs";
import {adaptiveTrace, allocation, plan} from "./adaptive_plan_test_support.mjs";

test("source failure reallocates the same range to an alternative source", () => {
  const primary = allocation({source: "http://127.0.0.1/v0-primary.mp4"});
  const mirror = allocation({source: "http://127.0.0.1/v0-mirror.mp4"});
  const trace = adaptiveTrace({
    scenario: "source_failure",
    adaptive_plans: [
      plan({allocations: [primary]}),
      plan({revision: 2, observed_at_ms: 200, allocations: [mirror]}),
    ],
  });

  assert.doesNotThrow(() => requireAdaptiveScenarioOutcome(trace));
});
