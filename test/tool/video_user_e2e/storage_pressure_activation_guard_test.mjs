import assert from "node:assert/strict";
import test from "node:test";
import {requireImpairmentActivation} from "../../../tool/video_user_e2e/impairment_activation.mjs";
import {activationTrace, storageSample} from "./impairment_activation_support.mjs";

const BUDGET = 2_097_152;

test("storage pressure must park near 2 MiB and resume after the budget release", () => {
  const trace = activationTrace("storage_pressure");
  trace.impairments = [storage(BUDGET), storage(67_108_864, 3_001)];
  trace.samples = [
    storageSample(1_000, 2_039_580),
    storageSample(1_200, 2_039_580),
    storageSample(2_900, 2_039_580),
    storageSample(3_200, 2_301_724),
  ];

  assert.doesNotThrow(() => requireImpairmentActivation(trace));

  trace.samples.at(-1).state.storage.used_bytes = 2_039_580;
  trace.origin_requests = [{
    method: "GET",
    started_at_ms: trace.started_at_epoch_ms + 3_100,
    bytes_sent: 32_768,
    completed: true,
  }];
  assert.doesNotThrow(() => requireImpairmentActivation(trace));

  trace.origin_requests = [];
  trace.samples.at(-1).state.storage.used_bytes = 2_039_580;
  assert.throws(
    () => requireImpairmentActivation(trace),
    /storage delivery did not resume after budget release/,
  );

  trace.samples.at(-1).state.storage.used_bytes = 2_301_724;
  trace.samples[1].state.storage.used_bytes = 1_900_000;
  trace.samples[2].state.storage.used_bytes = 1_900_000;
  assert.throws(
    () => requireImpairmentActivation(trace),
    /storage delivery did not park at the 2 MiB budget/,
  );

  trace.impairments = [];
  assert.throws(
    () => requireImpairmentActivation(trace),
    /storage pressure controls were not applied/,
  );
});

function storage(budget_bytes, at_ms) {
  return {kind: "storage", payload: {budget_bytes}, ...(at_ms ? {at_ms} : {})};
}
