import assert from "node:assert/strict";
import test from "node:test";
import {waitForWarmPrefetch} from "../../../tool/video_user_e2e/warm_prefetch.mjs";

const IDS = ["v0", "v1", "v2", "v3", "v4"];

test("a timed-out warm-prefetch poll retains its baseline and every sample", async () => {
  const trace = {warm_prefetch: null};
  const snapshots = [state({v0: 11, v4: 50}), state({v0: 21, v1: 12, v4: 75})];

  await assert.rejects(waitForWarmPrefetch({
    orderedIds: IDS,
    protectedCount: 4,
    baseline: state({v0: 1, v4: 25}),
    minimumBytes: 48,
    deadlineMs: 4_000,
    startedAt: 1_000,
    now: sequence(1_100, 1_200, 1_300),
    read: async () => snapshots.shift(),
    wait: async (input) => {
      await input.read();
      await input.read();
      throw new Error("simulated warm-prefetch timeout");
    },
    onEvidence: (evidence) => { trace.warm_prefetch = evidence; },
  }), /simulated warm-prefetch timeout/);

  assert.deepEqual(trace.warm_prefetch.baseline_bytes, {
    v0: 1, v1: 0, v2: 0, v3: 0, v4: 25,
  });
  assert.deepEqual(trace.warm_prefetch.samples, [
    {at_ms: 200, downloaded_bytes: {v0: 11, v1: 0, v2: 0, v3: 0, v4: 50}},
    {at_ms: 300, downloaded_bytes: {v0: 21, v1: 12, v2: 0, v3: 0, v4: 75}},
  ]);
});

function state(bytes) {
  return {videos: IDS.map((id) => ({id, downloaded_bytes: bytes[id] ?? 0}))};
}

function sequence(...values) {
  return () => values.shift();
}
