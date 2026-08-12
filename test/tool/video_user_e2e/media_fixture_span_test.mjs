import assert from "node:assert/strict";
import test from "node:test";
import {playableMedia} from "../../../tool/video_user_e2e/media_fixture.mjs";

const TRANSFER_CHUNK_BYTES = 64 * 1_024;
const LOW_BANDWIDTH_BPS = 700_000;
const STORAGE_BUDGET_BYTES = 2 * 1_024 * 1_024;
const VIDEO_COUNT = 8;

test("the playable fixture is sustainable under the drop and spans the storage budget", () => {
  const mdatType = playableMedia.bytes.indexOf(Buffer.from("mdat"));
  const mdatStart = mdatType - 4;
  const mdatSize = playableMedia.bytes.readUInt32BE(mdatStart);
  const bitrate = playableMedia.bytes.length * 8_000 / playableMedia.durationMs;

  assert.equal(playableMedia.durationMs, 6_000);
  assert.ok(bitrate < LOW_BANDWIDTH_BPS, {bitrate});
  assert.ok(playableMedia.bytes.length * VIDEO_COUNT > STORAGE_BUDGET_BYTES);
  assert.ok(playableMedia.bytes.length > 4 * TRANSFER_CHUNK_BYTES);
  assert.equal(mdatStart + mdatSize, playableMedia.bytes.length);
  assert.equal(sampleBytes(playableMedia.bytes), mdatSize - 8);
  assert.equal(mediaDurationMs(playableMedia.bytes), playableMedia.durationMs);
});

function sampleBytes(bytes) {
  const start = boxStart(bytes, "stsz");
  const count = bytes.readUInt32BE(start + 16);
  let total = 0;
  for (let index = 0; index < count; index += 1) {
    total += bytes.readUInt32BE(start + 20 + index * 4);
  }
  return total;
}

function mediaDurationMs(bytes) {
  const start = boxStart(bytes, "mdhd");
  const timescale = bytes.readUInt32BE(start + 20);
  return bytes.readUInt32BE(start + 24) * 1_000 / timescale;
}

function boxStart(bytes, type) {
  return bytes.indexOf(Buffer.from(type)) - 4;
}
