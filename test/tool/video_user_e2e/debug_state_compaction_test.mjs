import assert from "node:assert/strict";
import test from "node:test";
import {compactDebugState} from "../../../tool/video_user_e2e/debug_state_evidence.mjs";

test("sampled debug telemetry excludes duplicated adaptive plan history", () => {
  const state = {
    adaptive_plans: [{revision: 1}],
    decisions: {records: [{sequence: 1}]},
    evaluation: {presentation_samples: 1},
    videos: [{id: "v0"}],
    network: {bandwidth_kbps: 700},
  };

  assert.deepEqual(compactDebugState(state), {
    videos: [{id: "v0"}],
    network: {bandwidth_kbps: 700},
  });
  assert.deepEqual(state.adaptive_plans, [{revision: 1}]);
  assert.deepEqual(state.decisions.records, [{sequence: 1}]);
});
