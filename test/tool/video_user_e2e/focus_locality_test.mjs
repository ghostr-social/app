import assert from "node:assert/strict";
import test from "node:test";
import {createFocusLocalityClick} from "../../../tool/video_user_e2e/focus_locality.mjs";

const IDS = Array.from({length: 8}, (_, index) => `v${index}`);

test("focus locality snapshots shifting tail windows between exact origin boundaries", async () => {
  const trace = {};
  const requests = [{start_ordinal: 0, chunk_events: [{ordinal: 1}]}];
  const events = [];
  const click = createFocusLocalityClick({
    trace,
    orderedIds: IDS,
    protectedCount: 4,
    minimumBytes: 49_152,
    originRequests: requests,
    now: sequence(1_000, 2_000),
    read: async () => state(events.length),
    click: async (id) => {
      events.push(id);
      if (id === "v6") requests.push({start_ordinal: 2, chunk_events: [{ordinal: 3}]});
    },
  });

  await click("v6");
  await click("v7");

  assert.deepEqual(trace.focus_locality_epochs, [
    epoch({focus: "v6", protectedIds: ["v6", "v7"], baseline: 0,
      startedAfter: 1, startedAt: 1_000, endedAt: 2_000, endedThrough: 3}),
    epoch({focus: "v7", protectedIds: ["v7"], baseline: 1,
      startedAfter: 3, startedAt: 2_000}),
  ]);
});

test("focus locality rejects an ID outside the ordered feed", async () => {
  const click = createFocusLocalityClick({
    trace: {}, orderedIds: IDS, protectedCount: 4, minimumBytes: 49_152,
    originRequests: [], now: () => 1, read: async () => state(0), click: async () => {},
  });

  await assert.rejects(click("unknown"), /unknown video/);
});

function epoch(input) {
  return {focus_id: input.focus, protected_ids: input.protectedIds,
    started_after_origin_ordinal: input.startedAfter,
    started_at_epoch_ms: input.startedAt,
    baseline_bytes: Object.fromEntries(IDS.map((id) => [id, input.baseline])),
    minimum_bytes: 49_152,
    ...(input.endedAt ? {ended_at_epoch_ms: input.endedAt,
      ended_through_origin_ordinal: input.endedThrough} : {})};
}

function state(downloaded_bytes) {
  return {videos: IDS.map((id) => ({id, downloaded_bytes}))};
}

function sequence(...values) {
  return () => values.shift();
}
