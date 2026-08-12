import assert from "node:assert/strict";
import test from "node:test";
import {
  controlPoint, dispatchTrustedClick,
} from "../../../tool/video_user_e2e/page_runtime.mjs";
import {recordingPage} from "./page_runtime_support.mjs";

test("a visible control point drives one trusted mouse click", async () => {
  const point = {ready: true, x: 12, y: 34, label: "Play v0"};
  const {page, calls} = recordingPage((method) => {
    return method === "Runtime.evaluate" ? {result: {value: point}} : {};
  });

  assert.equal(await controlPoint(page, "v0"), point);
  await dispatchTrustedClick(page, point);

  assert.match(calls[0].params.expression, /video-row/);
  assert.deepEqual(calls.slice(1).map((call) => call.params.type), [
    "mouseMoved", "mousePressed", "mouseReleased",
  ]);
  assert.ok(calls.slice(1).every((call) => call.params.x === 12 && call.params.y === 34));
});
