import assert from "node:assert/strict";
import test from "node:test";
import {playerSnapshot} from "../../../tool/video_user_e2e/page_runtime.mjs";
import {recordingPage} from "./page_runtime_support.mjs";

test("the player snapshot reads visible playback facts", async () => {
  const player = {id: "v0", phase: "playing", current_time: 1.5};
  const {page, calls} = recordingPage({result: {value: player}});

  assert.equal(await playerSnapshot(page), player);
  assert.match(calls[0].params.expression, /player\.currentTime/);
  assert.match(calls[0].params.expression, /player\.error/);
  assert.match(calls[0].params.expression, /presentedFrame === true/);
});
