import assert from "node:assert/strict";
import test from "node:test";
import {
  requireReadyReserve,
} from "../../../tool/video_user_e2e/ready_reserve_acceptance.mjs";
import {
  plan, readyReserve,
} from "./adaptive_plan_test_support.mjs";

test("ready reserve counters match candidate states", () => {
  const structural = {post_id: "post-1", status: "structural"};
  const mismatchedStructural = plan({ready_reserve: readyReserve({
    ready: 0, structural: 0, protected: 1, candidates: [structural],
  })});
  const mismatchedProtected = plan({ready_reserve: readyReserve({
    ready: 0, structural: 1, protected: 0, candidates: [structural],
  })});

  assert.throws(() => requireReadyReserve(mismatchedStructural), /counters.*candidates/);
  assert.throws(() => requireReadyReserve(mismatchedProtected), /counters.*candidates/);
});
