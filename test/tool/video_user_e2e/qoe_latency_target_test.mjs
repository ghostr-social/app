import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe, requireQoeTargets} from "../../../tool/video_user_e2e/qoe_metrics.mjs";
import {QOE_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

test("startup and every visible focus switch meet their latency targets", () => {
  const trace = {
    clicks: [{id: "a", at_ms: 0}, {id: "b", at_ms: 1_000}],
    samples: [
      player("a", "starting", 100),
      player("a", "playing", 800),
      player("a", "playing", 900),
      player("b", "starting", 1_100),
      player("b", "playing", 1_600),
      player("b", "playing", 1_700),
    ],
  };

  const metrics = measureQoe(trace);

  assert.equal(metrics.startup_latency_ms, 800);
  assert.equal(metrics.focus_switch_latency_ms, 600);
  assert.doesNotThrow(() => requireQoeTargets(metrics, QOE_TARGETS));
});

function player(id, phase, at_ms) {
  const other = id === "a" ? "b" : "a";
  return {
    at_ms,
    player: {id, phase, current_time: 0},
    state: {videos: [
      {id, downloaded_bytes: 1_000, total_bytes: 4_000_000},
      {id: other, downloaded_bytes: 65_536, total_bytes: 4_000_000},
    ]},
  };
}
