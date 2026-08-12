import assert from "node:assert/strict";
import test from "node:test";
import {requireAdaptivePlanEvidence} from "../../../tool/video_user_e2e/adaptive_plan_acceptance.mjs";
import {
  adaptiveTrace, allocation, plan,
} from "./adaptive_plan_test_support.mjs";

test("adaptive plan evidence requires a reason and measurable playable gain", () => {
  assert.doesNotThrow(() => requireAdaptivePlanEvidence(adaptiveTrace()));

  const missingGain = adaptiveTrace({
    adaptive_plans: [plan({allocations: [allocation({expected_playable_gain_ms: 0})]})],
  });
  assert.throws(
    () => requireAdaptivePlanEvidence(missingGain),
    /expected playable gain/,
  );
});
