import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe} from "../../../tool/video_user_e2e/qoe_metrics.mjs";

const IDS = Array.from({length: 8}, (_, index) => `id${index}`);

test("moving QoE fails closed when pre-click origin boundary is missing", () => {
  const trace = {
    ordered_video_ids: IDS,
    video_ids: Object.fromEntries(IDS.map((id, index) => [`v${index}`, id])),
    origin_requests: [],
    focus_locality_epochs: [{
      pre_click: true, focus_id: IDS[0], protected_ids: IDS.slice(0, 4),
      started_at_epoch_ms: 1_000, baseline_bytes: bytes(), minimum_bytes: 49_152,
    }],
    clicks: [], samples: [],
  };

  assert.throws(() => measureQoe(trace), /focus locality ordinal is invalid/);
});

function bytes() {
  return Object.fromEntries(IDS.map((id) => [id, 0]));
}
