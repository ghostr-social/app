import assert from "node:assert/strict";
import test from "node:test";
import {OwnedLifecycle} from "../../../tool/video_user_e2e/lifecycle.mjs";

test("teardown stops only owned child groups in reverse order", async () => {
  const stopped = [];
  const lifecycle = new OwnedLifecycle({
    terminate: async (pid, signal) => stopped.push([pid, signal]),
  });
  lifecycle.track({pid: 101, label: "server"});
  lifecycle.track({pid: 202, label: "browser"});

  await lifecycle.teardown();
  await lifecycle.teardown();

  assert.deepEqual(stopped, [[202, "SIGTERM"], [101, "SIGTERM"]]);
  assert.throws(() => lifecycle.track({pid: 303}), /closed/);
});

test("unsafe process identifiers are rejected", () => {
  const lifecycle = new OwnedLifecycle();
  assert.throws(() => lifecycle.track({pid: 1}), /safe child PID/);
  assert.throws(() => lifecycle.track({pid: Number.NaN}), /safe child PID/);
});
