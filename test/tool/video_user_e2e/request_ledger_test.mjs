import assert from "node:assert/strict";
import test from "node:test";
import {RequestLedger} from "../../../tool/video_user_e2e/request_ledger.mjs";

test("request history is bounded and retains range response evidence", () => {
  const ledger = new RequestLedger({limit: 2});
  for (let index = 0; index < 3; index += 1) {
    ledger.request({
      requestId: String(index), timestamp: index,
      request: {url: `http://127.0.0.1/video/${index}`, method: "GET",
        headers: {Range: `bytes=${index}-`}},
    });
  }
  ledger.response({
    requestId: "2", timestamp: 4,
    response: {status: 206, mimeType: "video/mp4",
      headers: {"Content-Range": "bytes 2-9/10"}},
  });
  ledger.finished({requestId: "2", encodedDataLength: 8});

  assert.deepEqual(ledger.entries.map((entry) => entry.request_id), ["1", "2"]);
  assert.equal(ledger.entries.at(-1).range, "bytes=2-");
  assert.equal(ledger.entries.at(-1).content_range, "bytes 2-9/10");
  assert.equal(ledger.entries.at(-1).encoded_bytes, 8);
});
