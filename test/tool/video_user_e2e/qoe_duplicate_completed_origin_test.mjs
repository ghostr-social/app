import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe, requireQoeTargets} from "../../../tool/video_user_e2e/qoe_metrics.mjs";
import {QOE_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

test("only overlap among earlier successful completed source ranges is duplicate bandwidth", () => {
  const duplicate = measureQoe({origin_requests: [
    request({start: 0, end: 370_912, ordinal: 0}),
    request({start: 0, end: 370_912, ordinal: 1}),
  ]});
  const adjacent = measureQoe({origin_requests: [
    request({start: 0, end: 100, ordinal: 0}),
    request({start: 100, end: 200, ordinal: 1}),
  ]});
  const retry = measureQoe({origin_requests: [
    request({start: 0, end: 100, ordinal: 0, completed: false}),
    request({start: 0, end: 100, ordinal: 1}),
  ]});
  const failed = measureQoe({origin_requests: [
    request({start: 0, end: 100, ordinal: 0, failed_status: 503}),
    request({start: 0, end: 100, ordinal: 1}),
  ]});
  const injected = measureQoe({origin_requests: [
    request({start: 0, end: 100, ordinal: 0, injected_failure: true}),
    request({start: 0, end: 100, ordinal: 1}),
  ]});

  assert.equal(duplicate.duplicate_completed_origin_bytes, 370_912);
  assert.throws(() => requireQoeTargets(smooth(duplicate), QOE_TARGETS), /duplicate/);
  assert.equal(adjacent.duplicate_completed_origin_bytes, 0);
  assert.equal(retry.duplicate_completed_origin_bytes, 0);
  assert.equal(failed.duplicate_completed_origin_bytes, 0);
  assert.equal(injected.duplicate_completed_origin_bytes, 0);
});

function request(input) {
  return {id: "v6", start: input.start, end: input.end,
    start_ordinal: input.ordinal, completed: input.completed ?? true,
    injected_failure: input.injected_failure ?? false,
    failed_status: input.failed_status};
}

function smooth(metrics) {
  return {...metrics, startup_latency_ms: 0,
    focus_switch_latency_ms: 0, rebuffer_ratio: 0, cancellation_waste_bytes: 0,
    ahead_prefetch_bytes: 49_152};
}
