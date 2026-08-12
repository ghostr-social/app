import assert from "node:assert/strict";
import test from "node:test";
import {waitForWarmPrefetch} from "../../../tool/video_user_e2e/warm_prefetch.mjs";

const KIB = 1_024;
const FLOOR = 48 * KIB;
const IDS = Array.from({length: 8}, (_, index) => `v${index}`);

test("ordered protected videos gain their byte floor after the focus baseline", async () => {
  const calls = [];
  const evidence = await waitForWarmPrefetch({
    orderedIds: IDS,
    protectedCount: 4,
    baseline: state({v6: 64 * KIB}),
    minimumBytes: FLOOR,
    deadlineMs: 4_000,
    startedAt: 1_000,
    now: sequence(1_600, 2_000, 3_200),
    read: async () => state({v0: FLOOR, v1: FLOOR, v2: FLOOR, v3: FLOOR, v6: 64 * KIB}),
    wait: async (input) => {
      calls.push(input);
      const snapshot = await input.read();
      assert.equal(input.accept(snapshot), true);
      return snapshot;
    },
  });

  assert.equal(calls[0].timeoutMs, 3_400);
  assert.equal(evidence.latency_ms, 2_200);
  assert.equal(evidence.focus_started_at_epoch_ms, 1_000);
  assert.deepEqual(evidence.ready_bytes, {v0: FLOOR, v1: FLOOR, v2: FLOOR, v3: FLOOR});
  assert.equal(evidence.baseline_bytes.v6, 64 * KIB);
  assert.equal(evidence.samples[0].at_ms, 1_000);
  assert.equal(evidence.samples[0].downloaded_bytes.v6, 64 * KIB);
});

test("a ready protected frontier arriving after the deadline is rejected", async () => {
  await assert.rejects(waitForWarmPrefetch({
    orderedIds: IDS,
    protectedCount: 4,
    baseline: state({}),
    minimumBytes: FLOOR,
    deadlineMs: 4_000,
    startedAt: 1_000,
    now: sequence(1_200, 1_300, 5_001),
    read: async () => state({v0: FLOOR, v1: FLOOR, v2: FLOOR, v3: FLOOR}),
    wait: async (input) => input.read(),
  }), /warm prefetch latency 4001 exceeds 4000/);
});

function state(bytes) {
  return {videos: IDS.map((id) => ({id, downloaded_bytes: bytes[id] ?? 0}))};
}

function sequence(...values) {
  return () => values.shift();
}
