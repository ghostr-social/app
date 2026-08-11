import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe} from "../../../tool/video_user_e2e/qoe_metrics.mjs";

test("ahead-prefetch bytes include work completed before playback", () => {
  const trace = {
    warm_prefetch: {
      ordered_ids: ["a", "b", "c"],
      baseline_bytes: {a: 0, b: 0, c: 32},
      samples: [{downloaded_bytes: {a: 256, b: 128, c: 96}}],
      latency_ms: 10,
    },
    clicks: [],
    samples: [],
  };

  assert.equal(measureQoe(trace).ahead_prefetch_bytes, 192);
});
