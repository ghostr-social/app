import assert from "node:assert/strict";
import test from "node:test";
import {
  bootstrapImpairmentActions,
  playbackImpairmentActions,
} from "../../../tool/video_user_e2e/impairment_plan.mjs";

const PROTECTED_GRANTS = 4;
const PLAYBACK_SLICE_BYTES = 256 * 1_024;

test("bandwidth drop lands before protected startup delivery can finish", () => {
  const [initial] = bootstrapImpairmentActions("bandwidth_drop");
  const [drop, recovery] = playbackImpairmentActions("bandwidth_drop");
  const optimisticCompletionMs = PROTECTED_GRANTS * PLAYBACK_SLICE_BYTES * 8
    / initial.payload.bandwidth_kbps;

  assert.equal(optimisticCompletionMs, 3_355.4432);
  assert.ok(optimisticCompletionMs > drop.at_ms, {
    optimisticCompletionMs,
    dropAtMs: drop.at_ms,
  });
  assert.deepEqual(initial.payload, network(2_500));
  assert.deepEqual(drop, action(1_500, 700));
  assert.deepEqual(recovery, action(4_500, 2_500));
});

function action(at_ms, bandwidth_kbps) {
  return {at_ms, kind: "network", payload: network(bandwidth_kbps)};
}

function network(bandwidth_kbps) {
  return {bandwidth_kbps, latency_ms: 0, max_connections_per_host: 3};
}
