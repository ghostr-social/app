import assert from "node:assert/strict";
import test from "node:test";
import {requireImpairmentActivation} from "../../../tool/video_user_e2e/impairment_activation.mjs";
import {activationTrace, STARTED_AT} from "./impairment_activation_support.mjs";

test("packet loss must strike protected v2 twice before its long observation ends", () => {
  const trace = activationTrace("packet_loss");
  trace.origin_requests = [
    failure("v2", STARTED_AT + 1_000),
    failure("v2", STARTED_AT + 2_000),
  ];

  assert.doesNotThrow(() => requireImpairmentActivation(trace));

  trace.origin_requests[1] = failure("v5", STARTED_AT + 2_000);
  assert.throws(
    () => requireImpairmentActivation(trace),
    /packet loss did not inject two failures into clicked v2/,
  );

  trace.clicks[3].at_ms = 4_000;
  assert.throws(
    () => requireImpairmentActivation(trace),
    /packet loss did not observe clicked v2 for 2.5 seconds/,
  );
});

function failure(video, closed_at_ms) {
  return {video, injected_failure: true, closed_at_ms};
}
