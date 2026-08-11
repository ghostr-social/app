import assert from "node:assert/strict";
import test from "node:test";
import {
  measureOrderedPrefetch, requireOrderedPrefetchTargets,
} from "../../../tool/video_user_e2e/ordered_prefetch_acceptance.mjs";
import {ORDERED_PREFETCH_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

test("ordered prefetch readiness evidence fails closed", () => {
  assert.throws(
    () => measureOrderedPrefetch({ordered_video_ids: ["id0"], origin_requests: []}),
    /readiness evidence is missing/,
  );

  assert.throws(
    () => requireOrderedPrefetchTargets({
      protected_count: 4,
      protected_readiness_entries: 4,
      warm_prefetch_latency_ms: 4_000,
      protected_prefetch_min_bytes: 48 * 1_024 - 1,
      far_origin_body_bytes: 0,
      far_origin_request_starts: 0,
    }, ORDERED_PREFETCH_TARGETS),
    /protected bytes 49151 is below 49152/,
  );

  assert.throws(() => requireTrace(trace(1, ["id0"])),
    /protected count 1 does not equal 4/);
  assert.throws(() => requireTrace(trace(4, ["id0"])),
    /protected readiness entries 1 does not equal 4/);
});

function requireTrace(input) {
  return requireOrderedPrefetchTargets(measureOrderedPrefetch(input), ORDERED_PREFETCH_TARGETS);
}

function trace(protectedCount, readyIds) {
  const ids = Array.from({length: 8}, (_, index) => `id${index}`);
  return {ordered_video_ids: ids, video_ids: {}, origin_requests: [],
    warm_prefetch: {protected_count: protectedCount, latency_ms: 2_000,
      ready_bytes: Object.fromEntries(readyIds.map((id) => [id, 49_152]))}};
}
