import assert from "node:assert/strict";
import test from "node:test";
import {requireAdaptivePlanEvidence} from "../../../tool/video_user_e2e/adaptive_plan_acceptance.mjs";
import {
  adaptiveTrace, bodyRequest,
} from "./adaptive_plan_test_support.mjs";

test("every origin body range must have prior exact policy admission", () => {
  const admitted = adaptiveTrace({origin_requests: [
    bodyRequest(),
    bodyRequest({method: "HEAD", video: "v1"}),
  ]});
  assert.doesNotThrow(() => requireAdaptivePlanEvidence(admitted));

  const unadmitted = adaptiveTrace({origin_requests: [bodyRequest({video: "v1"})]});
  assert.throws(
    () => requireAdaptivePlanEvidence(unadmitted),
    /unadmitted origin range.*post-1/,
  );

  const rejectedProbe = adaptiveTrace({origin_requests: [bodyRequest({
    video: "v1", start: 0, end: 1, bytes_sent: 0, failed_status: 503,
  })]});
  assert.doesNotThrow(() => requireAdaptivePlanEvidence(rejectedProbe));
});
