import assert from "node:assert/strict";
import test from "node:test";
import {requireImpairmentActivation} from "../../../tool/video_user_e2e/impairment_activation.mjs";
import {activationTrace, networkState} from "./impairment_activation_support.mjs";

test("bandwidth drop must hit active incomplete delivery before recovery applies", () => {
  const trace = activationTrace("bandwidth_drop");
  trace.impairments = [
    network(700, 1_502, networkState(700)),
    network(2_500, 4_502, networkState(2_500)),
  ];

  assert.doesNotThrow(() => requireImpairmentActivation(trace));

  trace.impairments[0].after.connections[0].active = 0;
  assert.throws(
    () => requireImpairmentActivation(trace),
    /bandwidth drop was not applied during active incomplete delivery/,
  );

  trace.impairments.pop();
  assert.throws(
    () => requireImpairmentActivation(trace),
    /bandwidth recovery was not applied after the drop/,
  );
});

function network(bandwidth_kbps, at_ms, after) {
  return {kind: "network", payload: {bandwidth_kbps}, at_ms, after};
}
