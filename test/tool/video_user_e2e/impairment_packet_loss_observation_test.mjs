import assert from "node:assert/strict";
import test from "node:test";
import {
  playbackObservationSeconds,
} from "../../../tool/video_user_e2e/impairment_plan.mjs";

test("packet loss observes the twice-impaired protected video past a second gap", () => {
  assert.equal(playbackObservationSeconds("packet_loss", 2), 2.5);
  assert.equal(playbackObservationSeconds("packet_loss", 1), 0.75);
  assert.equal(playbackObservationSeconds(null, 2), 0.75);
});
