import assert from "node:assert/strict";
import test from "node:test";
import {requireImpairmentActivation} from "../../../tool/video_user_e2e/impairment_activation.mjs";
import {activationTrace, networkState} from "./impairment_activation_support.mjs";

test("protected transitions must sample their constrained network while delivery is active", () => {
  const trace = activationTrace("protected_transitions");
  trace.samples = [{state: networkState(2_500, 1, 100, 1)}];

  assert.doesNotThrow(() => requireImpairmentActivation(trace));

  trace.samples[0].state.connections[0].active = 0;
  assert.throws(
    () => requireImpairmentActivation(trace),
    /protected-transition profile was not sampled during active incomplete delivery/,
  );

  trace.samples[0].state = networkState(2_500, 1, 101, 1);
  assert.throws(() => requireImpairmentActivation(trace), /protected-transition profile/);

  trace.samples[0].state = networkState(2_500, 1, 100, 1);
  const video = trace.samples[0].state.videos[0];
  video.downloaded_bytes = video.total_bytes;
  assert.throws(() => requireImpairmentActivation(trace), /protected-transition profile/);
});
