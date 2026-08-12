import assert from "node:assert/strict";
import {EventEmitter} from "node:events";
import test from "node:test";
import {writeResponseChunk} from "../../../tool/video_user_e2e/response_chunk.mjs";

test("a backpressured origin chunk resolves on drain and rejects on close", async () => {
  const drained = response(false);
  const closed = response(false);
  const drainResult = writeResponseChunk(drained, Buffer.from("a"));
  const closeResult = writeResponseChunk(closed, Buffer.from("b"));

  drained.emit("drain");
  closed.emit("close");

  assert.equal(await drainResult, true);
  assert.equal(await closeResult, false);
  assert.equal(await writeResponseChunk(response(true), Buffer.from("c")), true);
  assert.equal(await writeResponseChunk(response(false, true), Buffer.from("d")), false);
  assert.equal(drained.listenerCount("close"), 0);
  assert.equal(closed.listenerCount("drain"), 0);
});

function response(writable, destroyed = false) {
  const value = new EventEmitter();
  value.destroyed = destroyed;
  value.write = () => writable;
  return value;
}
