import assert from "node:assert/strict";
import test from "node:test";
import {createVideoUserE2eRunner} from "../../../tool/video_user_e2e/runner.mjs";
import {successfulRunnerBoundaries} from "./runner_test_support.mjs";

test("protected transitions click the next video immediately after first playing", async () => {
  const fixture = successfulRunnerBoundaries();
  const refresh = fixture.boundaries.refreshDebugSnapshot;
  fixture.boundaries.refreshDebugSnapshot = async () => withNetwork(
    await refresh(), clickedCount(fixture.events),
  );
  fixture.boundaries.watchUntilPlaying = (input) => addPlaying(fixture.events, input);
  fixture.boundaries.watchProgress = (input) => addProgress(fixture.events, input);
  const run = createVideoUserE2eRunner(fixture.boundaries);

  const result = await run({
    root: "/tmp/video-runner", environment: {}, browser: {},
    scenario: "protected_transitions",
  });

  assert.deepEqual(journeyEvents(fixture.events), [
    "click:video-0", "start:video-0", "click:video-1", "start:video-1",
    "click:video-2", "start:video-2", "click:video-3", "progress:video-3",
  ]);
  assert.deepEqual(result.trace.clicks.map((click) => ({
    protected: click.protected_transition, transitionOnly: click.transition_only ?? false,
  })), [
    {protected: true, transitionOnly: true}, {protected: true, transitionOnly: true},
    {protected: true, transitionOnly: true}, {protected: true, transitionOnly: false},
  ]);
  assert.equal(result.trace.qoe.protected_transition_latency_ms, 1);
  assert.deepEqual(result.trace.focus_locality_epochs.map((epoch) => ({
    preClick: epoch.pre_click ?? false,
    focus: epoch.focus_id,
    protected: epoch.protected_ids,
    baseline: epoch.baseline_bytes[epoch.focus_id],
  })), [
    {preClick: true, focus: fixture.ids[0], protected: fixture.ids.slice(0, 4), baseline: 0},
    locality(fixture.ids, 0), locality(fixture.ids, 1),
    locality(fixture.ids, 2), locality(fixture.ids, 3),
  ]);
});

async function addPlaying(events, input) {
  events.push(`start:${input.id}`);
  input.trace.samples.push(sample(input, 1, 0));
  await pause();
}

async function addProgress(events, input) {
  events.push(`progress:${input.id}`);
  input.trace.samples.push(sample(input, 1, 0), sample(input, 751, 1));
  await pause();
}

function sample(input, elapsed, current_time) {
  const click = input.trace.clicks.at(-1);
  return {at_ms: click.at_ms + elapsed,
    player: {id: input.id, phase: "playing", current_time},
    state: withNetwork({videos: input.trace.ordered_video_ids.map((id) => ({
      id, downloaded_bytes: 65_536, total_bytes: 370_912,
    }))}, 0)};
}

function pause() {
  return new Promise((resolve) => setTimeout(resolve, 5));
}

function withNetwork(state, offset) {
  return {...state,
    videos: state.videos.map((video) => ({
      ...video, downloaded_bytes: video.downloaded_bytes + offset,
    })),
    network: {bandwidth_kbps: 2_500, latency_ms: 100, max_connections_per_host: 1},
    connections: [{host: "127.0.0.1", active: 1}]};
}

function clickedCount(events) {
  return events.filter((event) => event.startsWith("click:")).length;
}

function locality(ids, index) {
  return {preClick: false, focus: ids[index],
    protected: ids.slice(index, index + 4), baseline: 49_152 + index};
}

function journeyEvents(events) {
  return events.filter((event) => /^(click|start|progress):/.test(event));
}
