import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe} from "../../../tool/video_user_e2e/qoe_metrics.mjs";

const FLOOR = 48 * 1_024;
const IDS = ["v0", "v1", "v2", "v3", "v4"];

test("a sample that reaches the protected frontier does not charge later work", () => {
  const trace = {
    ordered_video_ids: IDS,
    warm_prefetch: {
      baseline_bytes: {v0: FLOOR, v1: FLOOR, v2: FLOOR, v3: 0, v4: 0},
      samples: [
        {downloaded_bytes: {v0: FLOOR, v1: FLOOR, v2: FLOOR, v3: 0, v4: 0}},
        {downloaded_bytes: {v0: FLOOR, v1: FLOOR, v2: FLOOR, v3: FLOOR, v4: FLOOR}},
      ],
      protected_count: 4,
      minimum_bytes: FLOOR,
      latency_ms: 10,
    },
    clicks: [],
    samples: [],
  };

  assert.equal(measureQoe(trace).far_ahead_before_frontier_bytes, 0);
});
