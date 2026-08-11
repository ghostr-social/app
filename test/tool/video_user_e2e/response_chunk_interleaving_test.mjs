import assert from "node:assert/strict";
import {EventEmitter} from "node:events";
import test from "node:test";
import {writeResponseChunk} from "../../../tool/video_user_e2e/response_chunk.mjs";

test("backpressure cannot invert the recorded order of concurrent chunk writes", async () => {
  const recorded = [];
  const blocked = response(false);
  const first = writeResponseChunk(
    blocked, Buffer.from("a"), () => recorded.push("first"),
  );
  const second = writeResponseChunk(
    response(true), Buffer.from("b"), () => recorded.push("second"),
  );

  assert.deepEqual(recorded, ["first", "second"]);
  blocked.emit("drain");
  assert.deepEqual(await Promise.all([first, second]), [true, true]);
});

function response(writable) {
  const value = new EventEmitter();
  value.destroyed = false;
  value.write = () => writable;
  return value;
}
