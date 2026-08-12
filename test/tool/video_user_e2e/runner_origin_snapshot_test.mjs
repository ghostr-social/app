import assert from "node:assert/strict";
import test from "node:test";
import {createVideoUserE2eRunner} from "../../../tool/video_user_e2e/runner.mjs";
import {successfulRunnerBoundaries} from "./runner_test_support.mjs";

test("the runner freezes origin evidence before measuring and writing artifacts", async () => {
  const fixture = successfulRunnerBoundaries();
  fixture.origin.requests.push({
    id: "v7", start: 0, end: 1, started_at_ms: 0, start_ordinal: 0,
    bytes_sent: 1, completed: false,
  });
  fixture.boundaries.writeSuccess = async () => {
    fixture.origin.requests[0].bytes_sent = 99;
  };
  const run = createVideoUserE2eRunner(fixture.boundaries);

  const result = await run({root: "/tmp/video-runner", environment: {}, browser: {}});

  assert.equal(result.trace.origin_requests[0].bytes_sent, 1);
  assert.notEqual(result.trace.origin_requests, fixture.origin.requests);
});
