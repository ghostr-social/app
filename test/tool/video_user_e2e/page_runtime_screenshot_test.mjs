import assert from "node:assert/strict";
import test from "node:test";
import {captureScreenshot} from "../../../tool/video_user_e2e/page_runtime.mjs";
import {recordingPage} from "./page_runtime_support.mjs";

test("a failure screenshot returns the encoded PNG", async () => {
  const {page, calls} = recordingPage({data: "encoded-png"});

  assert.equal(await captureScreenshot(page), "encoded-png");
  assert.deepEqual(calls[0], {
    method: "Page.captureScreenshot",
    params: {format: "png"},
    sessionId: "page-session",
  });
});
