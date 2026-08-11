import assert from "node:assert/strict";
import test from "node:test";
import {debugSnapshot} from "../../../tool/video_user_e2e/page_runtime.mjs";
import {recordingPage} from "./page_runtime_support.mjs";

test("the cached debug snapshot is read through the page session", async () => {
  const state = {videos: [{id: "v0"}]};
  const {page, calls} = recordingPage({result: {value: state}});

  assert.equal(await debugSnapshot(page), state);
  assert.match(calls[0].params.expression, /latestState/);
  assert.equal(calls[0].sessionId, "page-session");
});
