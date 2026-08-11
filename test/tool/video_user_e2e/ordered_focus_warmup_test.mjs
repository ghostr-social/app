import assert from "node:assert/strict";
import test from "node:test";
import {establishOrderedFocus} from "../../../tool/video_user_e2e/ordered_focus_warmup.mjs";

test("the warm baseline is captured before the first explicit focus", async () => {
  const events = [];
  const baseline = {videos: []};
  const session = await establishOrderedFocus({
    ids: ["v0", "v1"],
    now: () => 1_234,
    read: async () => {
      events.push("baseline");
      return baseline;
    },
    select: async (id) => { events.push(`focus:${id}`); },
    warm: async (timing) => {
      events.push("warm");
      assert.equal(timing.baseline, baseline);
      assert.equal(timing.startedAt, 1_234);
      return {ready: true};
    },
  });

  assert.deepEqual(events, ["baseline", "focus:v0", "warm"]);
  assert.equal(session.startedAt, 1_234);
  assert.deepEqual(await session.warm, {ready: true});
});
