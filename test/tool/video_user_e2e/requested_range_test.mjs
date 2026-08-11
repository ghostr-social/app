import assert from "node:assert/strict";
import test from "node:test";
import {requestedRange} from "../../../tool/video_user_e2e/requested_range.mjs";

test("origin ranges are single, bounded, and half open", () => {
  assert.deepEqual(requestedRange(undefined, 10), {
    start: 0,
    end: 10,
    partial: false,
  });
  assert.deepEqual(requestedRange("bytes=2-", 10), {
    start: 2,
    end: 10,
    partial: true,
  });
  assert.deepEqual(requestedRange("bytes=2-20", 10), {
    start: 2,
    end: 10,
    partial: true,
  });
  for (const invalid of ["items=0-1", "bytes=10-", "bytes=4-3", "bytes=0-NaN"]) {
    assert.equal(requestedRange(invalid, 10), null);
  }
});
