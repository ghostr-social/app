import assert from "node:assert/strict";
import test from "node:test";
import {waitForWarmPrefetch} from "../../../tool/video_user_e2e/warm_prefetch.mjs";

const FLOOR = 48 * 1_024;
const IDS = ["v0", "v1", "v2", "v3", "v4"];

test("protected bytes already present at focus count as transition readiness", async () => {
  const ready = state({v0: FLOOR, v1: FLOOR, v2: FLOOR, v3: FLOOR});
  const evidence = await waitForWarmPrefetch({
    orderedIds: IDS,
    protectedCount: 4,
    baseline: ready,
    minimumBytes: FLOOR,
    deadlineMs: 4_000,
    startedAt: 1_000,
    now: sequence(1_000, 1_010, 1_020),
    read: async () => ready,
    wait: async (input) => {
      const sample = await input.read();
      assert.equal(input.accept(sample), true);
      return sample;
    },
  });

  assert.deepEqual(evidence.ready_bytes, {v0: FLOOR, v1: FLOOR, v2: FLOOR, v3: FLOOR});
  assert.equal(evidence.latency_ms, 20);
});

function state(bytes) {
  return {videos: IDS.map((id) => ({id, downloaded_bytes: bytes[id] ?? 0}))};
}

function sequence(...values) {
  return () => values.shift();
}
