import assert from "node:assert/strict";
import test from "node:test";
import {requireImpairmentActivation} from "../../../tool/video_user_e2e/impairment_activation.mjs";
import {activationTrace, networkState} from "./impairment_activation_support.mjs";

test("high RTT must be sampled during active incomplete delivery", () => {
  const trace = activationTrace("high_rtt");
  trace.samples = [{state: networkState(2_500, 1, 450, 3)}];

  assert.doesNotThrow(() => requireImpairmentActivation(trace));

  trace.samples[0].state.network.latency_ms = 100;
  assert.throws(
    () => requireImpairmentActivation(trace),
    /high RTT profile was not sampled during active incomplete delivery/,
  );

  trace.samples[0].state = networkState(2_400, 1, 450, 3);
  assert.throws(() => requireImpairmentActivation(trace), /high RTT profile/);

  trace.samples[0].state = networkState(2_500, 1, 450, 2);
  assert.throws(() => requireImpairmentActivation(trace), /high RTT profile/);

  trace.impairments = [{after: networkState(2_500, 1, 450, 3)}];
  trace.samples = [];
  assert.throws(() => requireImpairmentActivation(trace), /high RTT profile/);
});
