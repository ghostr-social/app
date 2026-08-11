import assert from "node:assert/strict";
import test from "node:test";
import {
  bootstrapImpairmentActions,
  impairmentActions,
  impairmentOriginOptions,
  impairmentVideoRegistration,
  playbackImpairmentActions,
} from "../../../tool/video_user_e2e/impairment_plan.mjs";

test("impairment plans translate contracts into executable deterministic actions", () => {
  assert.deepEqual(impairmentOriginOptions("packet_loss"), {
    abortFirstAttempts: {video: "v2", count: 2},
    abortAfterBytes: 131_072,
  });
  assert.deepEqual(impairmentOriginOptions("source_failure"), {
    failSource: "primary",
  });
  assert.deepEqual(impairmentActions("high_rtt"), [{
    at_ms: 0,
    kind: "network",
    payload: {bandwidth_kbps: 2_500, latency_ms: 450, max_connections_per_host: 3},
  }]);
  assert.deepEqual(impairmentActions("storage_pressure"), [
    {at_ms: 0, kind: "storage", payload: {budget_bytes: 2_097_152}},
    {at_ms: 3_000, kind: "storage", payload: {budget_bytes: 67_108_864}},
  ]);
  assert.deepEqual(bootstrapImpairmentActions("high_rtt"), [
    {
      at_ms: 0,
      kind: "network",
      payload: {bandwidth_kbps: 2_500, latency_ms: 450, max_connections_per_host: 3},
    },
  ]);
  assert.deepEqual(bootstrapImpairmentActions("rapid_swipes"), []);
  assert.deepEqual(bootstrapImpairmentActions("protected_transitions"), [{
    at_ms: 0,
    kind: "network",
    payload: {
      bandwidth_kbps: 2_500,
      latency_ms: 100,
      max_connections_per_host: 1,
    },
  }]);
  assert.deepEqual(playbackImpairmentActions("high_rtt"), []);
  assert.deepEqual(playbackImpairmentActions("bandwidth_drop").map((step) => step.at_ms), [
    1_500,
    4_500,
  ]);

  assert.deepEqual(
    impairmentVideoRegistration("source_failure", "a", "http://127.0.0.1:42"),
    {
      url: "http://127.0.0.1:42/a-primary.mp4",
      mirrors: ["http://127.0.0.1:42/a-mirror.mp4"],
    },
  );
});
