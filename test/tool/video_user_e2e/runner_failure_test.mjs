import assert from "node:assert/strict";
import test from "node:test";
import {createVideoUserE2eRunner} from "../../../tool/video_user_e2e/runner.mjs";
import {successfulRunnerBoundaries} from "./runner_test_support.mjs";

test("the runner retains failure evidence and closes owned resources", async () => {
  const fixture = successfulRunnerBoundaries();
  fixture.boundaries.startServer = async () => { throw new Error("server failed"); };
  const run = createVideoUserE2eRunner(fixture.boundaries);

  await assert.rejects(
    () => run({root: "/tmp/video-runner", environment: {}, browser: {}}),
    /server failed; artifacts: \/tmp\/video-runner\/artifacts/,
  );

  assert.deepEqual(fixture.events, ["failure.write", "origin.close", "files.remove"]);
});
