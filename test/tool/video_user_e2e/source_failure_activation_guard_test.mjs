import assert from "node:assert/strict";
import test from "node:test";
import {requireImpairmentActivation} from "../../../tool/video_user_e2e/impairment_activation.mjs";
import {activationTrace, STARTED_AT} from "./impairment_activation_support.mjs";

test("source failure must route the selected video from primary 503 to mirror body", () => {
  const trace = activationTrace("source_failure");
  trace.origin_requests = [
    {id: "v0-primary", video: "v0", failed_status: 503,
      started_at_ms: STARTED_AT + 10, closed_at_ms: STARTED_AT + 11},
    {id: "v0-mirror", video: "v0", bytes_sent: 65_536, completed: true,
      started_at_ms: STARTED_AT + 12, closed_at_ms: STARTED_AT + 13},
  ];

  assert.doesNotThrow(() => requireImpairmentActivation(trace));

  trace.origin_requests[1].completed = false;
  assert.throws(
    () => requireImpairmentActivation(trace),
    /selected video did not complete a mirror body after primary 503/,
  );
});
