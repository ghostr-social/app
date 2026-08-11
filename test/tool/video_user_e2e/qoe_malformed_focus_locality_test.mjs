import assert from "node:assert/strict";
import test from "node:test";
import {measureQoe} from "../../../tool/video_user_e2e/qoe_metrics.mjs";

const IDS = ["id0", "id1", "id2", "id3", "id4"];

test("recorded focus locality rejects a noncontiguous protected window", () => {
  const trace = {
    ordered_video_ids: IDS,
    video_ids: Object.fromEntries(IDS.map((id, index) => [`v${index}`, id])),
    warm_prefetch: {
      focus_started_at_epoch_ms: 1_000,
      baseline_bytes: bytes(),
      samples: [], protected_count: 4, minimum_bytes: 49_152, latency_ms: 10,
    },
    focus_locality_epochs: [{
      focus_id: "id1", protected_ids: ["id1", "id3"],
      started_at_epoch_ms: 2_000, started_after_origin_ordinal: -1,
      baseline_bytes: bytes(), minimum_bytes: 49_152,
    }],
    origin_requests: [],
    clicks: [],
    samples: [],
  };

  assert.throws(() => measureQoe(trace), /focus locality protected IDs are invalid/);
});

function bytes() {
  return Object.fromEntries(IDS.map((id) => [id, 0]));
}
