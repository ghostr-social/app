import assert from "node:assert/strict";
import test from "node:test";
import {impairmentOriginOptions} from "../../../tool/video_user_e2e/impairment_plan.mjs";
import {IMPAIRMENT_SCENARIOS} from "../../../tool/video_user_e2e/impairment_scenarios.mjs";

test("packet loss targets the first two v2 body attempts", () => {
  assert.deepEqual(IMPAIRMENT_SCENARIOS.packet_loss.origin, {
    abort_first_attempts: {video: "v2", count: 2},
    abort_after_bytes: 65_536,
  });
  assert.deepEqual(impairmentOriginOptions("packet_loss"), {
    abortFirstAttempts: {video: "v2", count: 2},
    abortAfterBytes: 65_536,
  });
});
