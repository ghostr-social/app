import assert from "node:assert/strict";
import test from "node:test";
import {runImpairmentJourney} from "../../../tool/video_user_e2e/impairment_journey.mjs";
import {playbackImpairmentActions} from "../../../tool/video_user_e2e/impairment_plan.mjs";

test("rapid swipes supersede intermediate clicks and settle the final player", async () => {
  let now = 1_000;
  const clicked = [];
  const watched = [];
  const trace = {clicks: []};
  const actions = playbackImpairmentActions("rapid_swipes");

  await runImpairmentJourney({
    actions,
    ids: ["a", "b", "c", "d"],
    trace,
    startedAt: 1_000,
    now: () => now,
    wait: async (milliseconds) => now += milliseconds,
    click: async (id) => {
      clicked.push(id);
      now += 50;
    },
    watch: async (id) => watched.push(id),
    send: async () => {},
  });

  assert.deepEqual(clicked, ["a", "b", "c", "d"]);
  assert.deepEqual(watched, ["d"]);
  assert.deepEqual(trace.clicks, [
    {id: "a", at_ms: 0, superseded: true},
    {id: "b", at_ms: 200, superseded: true},
    {id: "c", at_ms: 400, superseded: true},
    {id: "d", at_ms: 600, superseded: false},
  ]);
});

test("standard impairments stay observable through the last scheduled transition", async () => {
  let now = 1_000;
  const clicked = [];
  const watched = [];
  const trace = {clicks: []};

  await runImpairmentJourney({
    actions: [],
    ids: ["v0", "v1", "v2", "v3", "v4", "v5", "v6", "v7"],
    trace,
    startedAt: now,
    now: () => now,
    click: async (id) => {
      clicked.push(id);
      now += 10;
    },
    watch: async (id) => {
      watched.push(id);
      now += 750;
    },
    send: async () => {},
  });

  assert.deepEqual(clicked, ["v0", "v1", "v2", "v3"]);
  assert.deepEqual(watched, clicked);
  assert.ok(now - 1_000 > 3_000);
});
