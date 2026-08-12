import assert from "node:assert/strict";
import test from "node:test";
import {requireAdaptiveScenarioOutcome} from "../../../tool/video_user_e2e/adaptive_scenario_acceptance.mjs";
import {
  adaptiveTrace, allocation, bodyRequest, plan,
} from "./adaptive_plan_test_support.mjs";

test("degraded network changes cost without restarting an already narrow plan", () => {
  const current = allocation({post_id: "post-1"});
  const retained = [{post_id: "post-0", range: allocation().range,
    source: allocation().source, committed_until_ms: 5_000, reason: "useful_commitment"}];
  const trace = adaptiveTrace({
    scenario: "bandwidth_drop",
    impairments: [receipt(200, 700), receipt(400, 2_500)],
    adaptive_plans: [
      plan({allocations: [current], retained}),
      plan({revision: 2, observed_at_ms: 250, allocations: [allocation({
        post_id: "post-1",
        utility: {...current.utility, expected_delivery_ms: 175},
      })], retained}),
    ],
  });

  assert.doesNotThrow(() => requireAdaptiveScenarioOutcome(trace));
  assert.throws(() => requireAdaptiveScenarioOutcome({
    ...trace,
    origin_requests: [request(220), request(300)],
  }), /restarted useful origin bytes/);
});

test("degraded network updates cost evidence on preserved exact work", () => {
  const trace = adaptiveTrace({
    scenario: "bandwidth_drop",
    impairments: [receipt(200, 700), receipt(400, 2_500)],
    adaptive_plans: [
      plan({allocations: [], retained: [retained(100)]}),
      plan({revision: 2, observed_at_ms: 250,
        allocations: [], retained: [retained(250)]}),
    ],
  });

  assert.doesNotThrow(() => requireAdaptiveScenarioOutcome(trace));
});

test("the next focus may flush exact-work network cost evidence", () => {
  const work = allocation();
  const changed = {...work, utility: {...work.utility, expected_delivery_ms: 175}};
  const trace = adaptiveTrace({
    scenario: "bandwidth_drop",
    started_at_epoch_ms: 0,
    clicks: [{id: "post-1", at_ms: 201}],
    impairments: [receipt(200, 700), receipt(400, 2_500)],
    adaptive_plans: [
      plan({allocations: [work]}),
      plan({revision: 2, observed_at_ms: 200, allocations: [work]}),
      plan({revision: 3, observed_at_ms: 250, allocations: [changed]}),
    ],
  });

  assert.doesNotThrow(() => requireAdaptiveScenarioOutcome(trace));
});

test("network replanning excludes canceled bytes from completed-range overlap", () => {
  const work = allocation({post_id: "post-1"});
  const trace = adaptiveTrace({
    scenario: "bandwidth_drop",
    impairments: [receipt(200, 700), receipt(400, 2_500)],
    adaptive_plans: [
      plan({allocations: [work]}),
      plan({revision: 2, observed_at_ms: 250,
        allocations: [{...work, utility: {...work.utility,
          expected_delivery_ms: 175}}]}),
    ],
    origin_requests: [
      {...request(220), completed: false, canceled: true},
      request(300),
    ],
  });

  assert.doesNotThrow(() => requireAdaptiveScenarioOutcome(trace));
});

function receipt(applied_at_epoch_ms, bandwidth_kbps) {
  return {kind: "network", applied_at_epoch_ms, payload: {bandwidth_kbps}};
}

function request(started_at_ms) {
  return bodyRequest({video: "v1", started_at_ms, bytes_sent: 32 * 1_024,
    completed: true, canceled: false});
}

function retained(expected_delivery_ms) {
  const work = allocation();
  return {post_id: work.post_id, range: work.range, source: work.source,
    utility: {...work.utility, expected_delivery_ms},
    committed_until_ms: work.commitment_until_ms, reason: "useful_commitment"};
}
