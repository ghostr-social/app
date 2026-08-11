import assert from "node:assert/strict";
import test from "node:test";
import {sendControlAction} from "../../../tool/video_user_e2e/impairment_executor.mjs";

test("supported impairment controls use their loopback PUT endpoints", async () => {
  const calls = [];
  const request = async (url, options) => {
    calls.push({url, ...options, body: JSON.parse(options.body)});
    return {ok: true};
  };

  await sendControlAction("http://127.0.0.1:42", action("network"), request);
  await sendControlAction("http://127.0.0.1:42", action("storage"), request);

  assert.deepEqual(calls.map((call) => [call.url, call.method, call.body]), [
    ["http://127.0.0.1:42/api/network", "PUT", {value: 1}],
    ["http://127.0.0.1:42/api/storage", "PUT", {value: 1}],
  ]);
});

function action(kind) {
  return {kind, payload: {value: 1}};
}
