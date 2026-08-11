import assert from "node:assert/strict";
import test from "node:test";
import {executeTimedActions} from "../../../tool/video_user_e2e/impairment_executor.mjs";

test("scheduled impairment actions execute once in timestamp order", async () => {
  let now = 1_000;
  const waits = [];
  const sent = [];
  const actions = [
    {at_ms: 0, kind: "network", payload: {bandwidth_kbps: 8_000}},
    {at_ms: 1_500, kind: "network", payload: {bandwidth_kbps: 700}},
    {at_ms: 4_500, kind: "network", payload: {bandwidth_kbps: 2_500}},
  ];

  await executeTimedActions({
    actions,
    startedAt: 1_000,
    clock: () => now,
    wait: async (milliseconds) => {
      waits.push(milliseconds);
      now += milliseconds;
    },
    send: async (action) => sent.push(action.payload.bandwidth_kbps),
  });

  assert.deepEqual(waits, [0, 1_500, 3_000]);
  assert.deepEqual(sent, [8_000, 700, 2_500]);
});
