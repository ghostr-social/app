import assert from "node:assert/strict";
import test from "node:test";
import {validateJourney} from "../../../tool/video_user_e2e/journey_outcome.mjs";

test("the journey rejects media without truthful partial-range evidence", () => {
  const trace = {requests: [{
    url: "http://127.0.0.1/video.mp4", method: "GET", status: 200,
  }]};

  assert.throws(() => validateJourney(trace), /truthful Range\/206 response/);
});
