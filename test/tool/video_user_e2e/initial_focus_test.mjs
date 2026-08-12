import assert from "node:assert/strict";
import test from "node:test";
import {establishInitialFocus} from "../../../tool/video_user_e2e/initial_focus.mjs";

test("initial focus selects the first admitted item and records its start", async () => {
  const selected = [];
  const result = await establishInitialFocus({
    ids: ["v0", "v1"],
    now: () => 1_234,
    select: async (id) => selected.push(id),
  });

  assert.deepEqual(selected, ["v0"]);
  assert.equal(result.startedAt, 1_234);
});
