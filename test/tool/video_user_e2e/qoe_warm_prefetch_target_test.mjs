import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe, requireQoeTargets} from "../../../tool/video_user_e2e/qoe_metrics.mjs";
import {QOE_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

test("warm-prefetch admission has an explicit four-second QoE deadline", () => {
  assert.equal(QOE_TARGETS.warm_prefetch_latency_ms, 4_000);
  assert.doesNotThrow(() => requireQoeTargets(metrics(4_000), QOE_TARGETS));
  assert.throws(
    () => requireQoeTargets(metrics(4_001), QOE_TARGETS),
    /warm-prefetch readiness/,
  );
});

test("warm-prefetch trace evidence is retained in measured QoE", () => {
  const measured = measureQoe({warm_prefetch: {latency_ms: 2_345}});

  assert.equal(measured.warm_prefetch_latency_ms, 2_345);
});

function metrics(warm_prefetch_latency_ms) {
  return {
    warm_prefetch_latency_ms,
    startup_latency_ms: 100,
    focus_switch_latency_ms: 100,
    rebuffer_ratio: 0,
    cancellation_waste_bytes: 0,
    ahead_prefetch_bytes: 65_536,
    far_ahead_before_frontier_bytes: 0,
    far_ahead_request_starts_before_frontier: 0,
    duplicate_completed_origin_bytes: 0,
    protected_transition_latency_ms: 0,
  };
}
