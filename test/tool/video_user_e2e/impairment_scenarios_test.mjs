import assert from "node:assert/strict";
import test from "node:test";
import {IMPAIRMENT_SCENARIOS} from "../../../tool/video_user_e2e/impairment_scenarios.mjs";

test("the browser suite deterministically covers every required failure mode", () => {
  assert.deepEqual(Object.keys(IMPAIRMENT_SCENARIOS), [
    "adaptive_plans",
    "bandwidth_drop",
    "packet_loss",
    "high_rtt",
    "rapid_swipes",
    "storage_pressure",
    "source_failure",
    "protected_transitions",
  ]);
  assert.deepEqual(IMPAIRMENT_SCENARIOS.adaptive_plans, {});
  assert.deepEqual(IMPAIRMENT_SCENARIOS.bandwidth_drop.network.steps, [
    {at_ms: 0, bandwidth_kbps: 2_500},
    {at_ms: 1_500, bandwidth_kbps: 700},
    {at_ms: 4_500, bandwidth_kbps: 2_500},
  ]);
  assert.deepEqual(IMPAIRMENT_SCENARIOS.packet_loss.origin.abort_first_attempts, {
    video: "v2",
    count: 2,
  });
  assert.equal(IMPAIRMENT_SCENARIOS.packet_loss.network.steps[1].packet_loss_bps, 6_000);
  assert.equal(IMPAIRMENT_SCENARIOS.high_rtt.network.latency_ms, 450);
  assert.deepEqual(IMPAIRMENT_SCENARIOS.rapid_swipes.focus, [
    {at_ms: 0, index: 0},
    {at_ms: 200, index: 1},
    {at_ms: 400, index: 2},
    {at_ms: 600, index: 3},
  ]);
  assert.equal(IMPAIRMENT_SCENARIOS.rapid_swipes.network.bandwidth_kbps, 2_500);
  assert.equal(IMPAIRMENT_SCENARIOS.storage_pressure.storage.release_at_ms, 3_000);
  assert.equal(IMPAIRMENT_SCENARIOS.source_failure.origin.fail_source, "primary");
  assert.equal(IMPAIRMENT_SCENARIOS.source_failure.network.bandwidth_kbps, 2_500);
  assert.deepEqual(IMPAIRMENT_SCENARIOS.protected_transitions.network, {
    bandwidth_kbps: 2_500,
    latency_ms: 100,
    max_connections_per_host: 1,
  });
});
