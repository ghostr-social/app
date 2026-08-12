import assert from "node:assert/strict";
import test from "node:test";
import {runVideoE2eMain} from "../../../tool/video_user_e2e/main.mjs";

test("the main adapter runs the selected impairment and reports artifacts", async () => {
  const logs = [];
  let input;

  await runVideoE2eMain({
    arguments: ["--scenario=packet_loss"],
    environment: {TOKEN: "local"},
    root: "/workspace",
    verify: async () => ({version: "Server Chromium 1", path: "/browser"}),
    run: async (value) => { input = value; return {artifacts: "/artifacts/run-1"}; },
    log: (message) => logs.push(message),
  });

  assert.equal(input.scenario, "packet_loss");
  assert.equal(input.root, "/workspace");
  assert.equal(input.environment.TOKEN, "local");
  assert.equal(logs.at(-1), "Local video user E2E passed; artifacts: /artifacts/run-1");
});
