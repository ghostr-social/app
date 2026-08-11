import assert from "node:assert/strict";
import test from "node:test";
import {ORDERED_PREFETCH_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

test("ordered prefetch owns unchanged locality and readiness targets", () => {
  assert.deepEqual(ORDERED_PREFETCH_TARGETS, {
    protected_count: 4,
    minimum_bytes: 48 * 1_024,
    latency_ms: 4_000,
    far_origin_body_bytes: 0,
    far_origin_request_starts: 0,
  });
});
