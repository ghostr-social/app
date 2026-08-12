import assert from "node:assert/strict";
import test from "node:test";
import {RequestLedger} from "../../../tool/video_user_e2e/request_ledger.mjs";

test("request history retains body completion and failure evidence", () => {
  const ledger = new RequestLedger();
  ledger.request({requestId: "ok", timestamp: 1,
    request: {url: "http://127.0.0.1/video.mp4", method: "GET", headers: {}}});
  ledger.finished({requestId: "ok", encodedDataLength: 8});
  ledger.request({requestId: "bad", timestamp: 2,
    request: {url: "http://127.0.0.1/video.mp4", method: "GET", headers: {}}});

  ledger.failed({requestId: "bad", errorText: "net::ERR_FAILED", canceled: false});

  assert.equal(ledger.entries[0].finished, true);
  assert.equal(ledger.entries[1].failure, "net::ERR_FAILED");
  assert.equal(ledger.entries[1].canceled, false);
});
