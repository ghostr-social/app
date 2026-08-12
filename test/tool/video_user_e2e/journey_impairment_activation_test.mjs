import assert from "node:assert/strict";
import test from "node:test";
import {validateJourney} from "../../../tool/video_user_e2e/journey_outcome.mjs";

test("journey acceptance fails closed when its named impairment was irrelevant", () => {
  const trace = {
    scenario: "source_failure",
    video_ids: {v0: "selected"},
    clicks: [{id: "selected", at_ms: 0}],
    origin_requests: [],
    impairments: [],
    requests: [],
    samples: [],
  };

  assert.throws(
    () => validateJourney(trace),
    /selected video did not complete a mirror body after primary 503/,
  );
});
