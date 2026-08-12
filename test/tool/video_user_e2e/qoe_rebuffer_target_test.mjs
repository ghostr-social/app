import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe, requireQoeTargets} from "../../../tool/video_user_e2e/qoe_metrics.mjs";
import {QOE_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

test("rebuffer time is measured after startup and rejected above one percent", () => {
  const trace = {
    warm_prefetch: {latency_ms: 1_000},
    clicks: [{id: "a", at_ms: 0}],
    samples: [
      player("a", "starting", 0),
      player("a", "playing", 100),
      player("a", "stalled", 1_000),
      player("a", "playing", 1_100),
      player("a", "playing", 10_100),
    ],
  };

  const metrics = measureQoe(trace);

  assert.equal(metrics.rebuffer_duration_ms, 100);
  assert.equal(metrics.observed_playback_ms, 10_000);
  assert.equal(metrics.rebuffer_ratio, 0.01);
  assert.doesNotThrow(() => requireQoeTargets(metrics, QOE_TARGETS));
  assert.throws(
    () => requireQoeTargets({...metrics, rebuffer_ratio: 0.011}, QOE_TARGETS),
    /rebuffer ratio/,
  );
});

function player(id, phase, at_ms) {
  return {
    at_ms,
    player: {id, phase, current_time: 0},
    state: {videos: [
      {id, downloaded_bytes: 1_000, total_bytes: 4_000_000},
      {id: "b", downloaded_bytes: 65_536, total_bytes: 4_000_000},
    ]},
  };
}
