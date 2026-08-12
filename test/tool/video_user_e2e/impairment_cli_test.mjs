import assert from "node:assert/strict";
import test from "node:test";
import {parseVideoE2eArguments} from "../../../tool/video_user_e2e/arguments.mjs";

test("the CLI accepts one known impairment and rejects unknown or duplicate input", () => {
  assert.deepEqual(parseVideoE2eArguments([]), {checkOnly: false, scenario: null});
  assert.deepEqual(parseVideoE2eArguments(["--check-prerequisites"]), {
    checkOnly: true,
    scenario: null,
  });
  assert.deepEqual(parseVideoE2eArguments(["--scenario=packet_loss"]), {
    checkOnly: false,
    scenario: "packet_loss",
  });
  assert.equal(
    parseVideoE2eArguments(["--scenario=adaptive_plans"]).scenario,
    "adaptive_plans",
  );
  assert.throws(() => parseVideoE2eArguments(["--scenario=random"]), /unknown scenario/);
  assert.throws(
    () => parseVideoE2eArguments(["--scenario=high_rtt", "--scenario=packet_loss"]),
    /one scenario/,
  );
});
