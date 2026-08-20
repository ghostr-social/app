import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe} from "../../../tool/video_user_e2e/qoe_metrics.mjs";

test("swipe latency quantiles require explicit presented-frame samples", () => {
  const trace = {
    clicks: [{id: "a", at_ms: 0}, {id: "b", at_ms: 1_000}],
    samples: [
      sample("a", "playing", false, 100),
      sample("a", "playing", true, 300),
      sample("b", "playing", false, 1_100),
      sample("b", "playing", true, 1_900),
    ],
  };

  const metrics = measureQoe(trace);

  assert.deepEqual(metrics.swipe_to_first_frame_ms, [300, 900]);
  assert.equal(metrics.swipe_to_first_frame_p50_ms, 300);
  assert.equal(metrics.swipe_to_first_frame_p95_ms, 900);
  assert.equal(metrics.swipe_to_first_frame_p99_ms, 900);
  assert.equal(metrics.startup_failure_rate, 0);
});

test("playing without a presented frame remains a startup failure", () => {
  const metrics = measureQoe({
    clicks: [{id: "a", at_ms: 0}],
    samples: [sample("a", "playing", false, 200)],
  });

  assert.deepEqual(metrics.swipe_to_first_frame_ms, []);
  assert.equal(metrics.swipe_to_first_frame_p99_ms, Number.POSITIVE_INFINITY);
  assert.equal(metrics.startup_failure_rate, 1);
  assert.equal(metrics.startup_latency_ms, Number.POSITIVE_INFINITY);
});

function sample(id, phase, presented, at_ms) {
  return {
    at_ms,
    player: {id, phase, presented, current_time: 0},
    state: {videos: []},
  };
}
