import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe, requireQoeTargets} from "../../../tool/video_user_e2e/qoe_metrics.mjs";
import {QOE_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

test("every tagged protected transition must play within five hundred milliseconds", () => {
  const atTarget = measureQoe(trace(500));
  const late = measureQoe(trace(501));
  const missing = measureQoe({clicks: [click(0)], samples: []});

  assert.equal(atTarget.protected_transition_latency_ms, 500);
  assert.doesNotThrow(() => requireQoeTargets(smooth(atTarget), QOE_TARGETS));
  assert.throws(() => requireQoeTargets(smooth(late), QOE_TARGETS), /protected transition/);
  assert.equal(missing.protected_transition_latency_ms, Number.POSITIVE_INFINITY);
  assert.throws(() => requireQoeTargets(smooth(missing), QOE_TARGETS), /protected transition/);
});

function trace(latency) {
  return {clicks: [click(0)], samples: [
    sample(latency, 0), sample(latency + 100, 1),
  ]};
}

function click(at_ms) {
  return {id: "v0", at_ms, protected_transition: true};
}

function sample(at_ms, current_time) {
  return {at_ms, player: {id: "v0", phase: "playing", current_time}, state: {videos: []}};
}

function smooth(metrics) {
  return {...metrics, warm_prefetch_latency_ms: 0, startup_latency_ms: 0,
    focus_switch_latency_ms: 0, rebuffer_ratio: 0, cancellation_waste_bytes: 0,
    ahead_prefetch_bytes: 49_152, far_ahead_before_frontier_bytes: 0,
    far_ahead_request_starts_before_frontier: 0, duplicate_completed_origin_bytes: 0};
}
