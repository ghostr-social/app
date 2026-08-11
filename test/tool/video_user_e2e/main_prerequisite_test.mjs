import assert from "node:assert/strict";
import test from "node:test";
import {launchVideoE2eMain} from "../../../tool/video_user_e2e/main.mjs";

test("the main adapter stops after a successful prerequisite check", async () => {
  const logs = [];
  let ranJourney = false;

  await launchVideoE2eMain({
    direct: true,
    main: {
      arguments: ["--check-prerequisites"],
      environment: {},
      root: "/workspace",
      verify: async () => ({version: "1", sha256: "abc"}),
      run: async () => { ranJourney = true; },
      log: (message) => logs.push(message),
    },
  });

  assert.equal(ranJourney, false);
  assert.deepEqual(logs, ["Pinned browser verified: 1 (abc)"]);
});
