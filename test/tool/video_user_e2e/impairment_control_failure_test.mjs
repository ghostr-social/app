import assert from "node:assert/strict";
import test from "node:test";
import {sendControlAction} from "../../../tool/video_user_e2e/impairment_executor.mjs";

test("unsupported and rejected impairment controls fail closed", async () => {
  const rejected = () => sendControlAction(
    "http://127.0.0.1:42",
    {kind: "network", payload: {}},
    async () => ({ok: false, status: 503}),
  );
  const unsupported = () => sendControlAction(
    "http://127.0.0.1:42", {kind: "unknown", payload: {}},
  );

  await assert.rejects(rejected, /network impairment failed: HTTP 503/);
  await assert.rejects(unsupported, /unsupported impairment action: unknown/);
});
