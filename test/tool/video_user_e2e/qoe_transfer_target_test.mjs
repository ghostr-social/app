import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe, requireQoeTargets} from "../../../tool/video_user_e2e/qoe_metrics.mjs";
import {QOE_TARGETS} from "../../../tool/video_user_e2e/qoe_targets.mjs";

test("adaptive delivery bounds waste and speculative bytes without forcing a floor", () => {
  const trace = {
    clicks: [{id: "a", at_ms: 0}],
    samples: [sample(100, 256_000, 65_536), sample(900, 512_000, 131_072)],
    origin_requests: [
      {id: "a", bytes_sent: 131_072, completed: false, canceled: true},
      {id: "b", bytes_sent: 65_536, completed: true, canceled: false},
    ],
  };

  const metrics = measureQoe(trace);

  assert.equal(metrics.cancellation_waste_bytes, 131_072);
  assert.equal(metrics.ahead_prefetch_bytes, 131_072);
  assert.doesNotThrow(() => requireQoeTargets(metrics, QOE_TARGETS));
  assert.doesNotThrow(() => requireQoeTargets({
    ...metrics,
    ahead_prefetch_bytes: 0,
  }, QOE_TARGETS));
  assert.throws(
    () => requireQoeTargets({...metrics, ahead_prefetch_bytes: 3 * 1_024 * 1_024 + 1},
      QOE_TARGETS),
    /ahead prefetch/,
  );
});

function sample(at_ms, current, ahead) {
  return {
    at_ms,
    player: {id: "a", phase: "playing", current_time: at_ms / 1_000},
    state: {videos: [
      {id: "a", downloaded_bytes: current, total_bytes: 4_000_000},
      {id: "b", downloaded_bytes: ahead, total_bytes: 4_000_000},
    ]},
  };
}
