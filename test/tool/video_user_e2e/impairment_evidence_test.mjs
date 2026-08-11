import assert from "node:assert/strict";
import test from "node:test";
import {createEvidenceSender} from "../../../tool/video_user_e2e/impairment_evidence.mjs";

test("an impairment receipt retains application time and surrounding debug state", async () => {
  const evidence = [];
  const states = [{network: {bandwidth_kbps: 8_000}}, {network: {bandwidth_kbps: 700}}];
  const sent = [];
  const send = createEvidenceSender({
    evidence,
    startedAt: 10_000,
    now: () => 11_502,
    read: async () => states.shift(),
    send: async (action) => sent.push(action),
  });
  const action = {kind: "network", payload: {bandwidth_kbps: 700}};

  await send(action);

  assert.deepEqual(sent, [action]);
  assert.deepEqual(evidence, [{
    kind: "network",
    payload: {bandwidth_kbps: 700},
    applied_at_epoch_ms: 11_502,
    at_ms: 1_502,
    before: {network: {bandwidth_kbps: 8_000}},
    after: {network: {bandwidth_kbps: 700}},
  }]);

  const bootstrap = [];
  const applyBootstrap = createEvidenceSender({
    evidence: bootstrap,
    now: () => 9_000,
    send: async () => {},
  });
  await applyBootstrap(action);
  assert.deepEqual(bootstrap, [{
    kind: "network",
    payload: {bandwidth_kbps: 700},
    applied_at_epoch_ms: 9_000,
  }]);
});
