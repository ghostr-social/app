import assert from "node:assert/strict";
import test from "node:test";
import {createVideoUserE2eRunner} from "../../../tool/video_user_e2e/runner.mjs";
import {successfulRunnerBoundaries} from "./runner_test_support.mjs";

test("ordered prefetch holds initial focus without warming later transitions", async () => {
  const fixture = successfulRunnerBoundaries();
  const run = createVideoUserE2eRunner(fixture.boundaries);

  const result = await run({root: "/tmp/video-runner", environment: {}, browser: {},
    scenario: "ordered_prefetch"});

  assert.deepEqual(result.trace.clicks, []);
  assert.equal(result.trace.warm_prefetch.ready_bytes[fixture.ids[3]], 49_152);
  assert.equal(result.trace.qoe.far_origin_body_bytes, 0);
  assert.equal(result.trace.qoe.far_origin_request_starts, 0);
  assert.equal(fixture.events.some((event) => event.startsWith("click:")), false);
});
