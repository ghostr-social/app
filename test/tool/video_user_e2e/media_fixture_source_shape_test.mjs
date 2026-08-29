import assert from "node:assert/strict";
import {createHash} from "node:crypto";
import test from "node:test";
import {sourceMp4} from "../../../tool/video_user_e2e/media_fixture.mjs";

const SOURCE_SHA256 =
  "ebfb2821b5362cfc4f0791c7bc47b7c9023f6ff04b3a17ec64a3a5ed83008344";

test("the source MP4 is a seekable visible-motion fixture", () => {
  const stsz = boxStart("stsz");
  const count = sourceMp4.readUInt32BE(stsz + 16);
  const sizes = Array.from(
    {length: count},
    (_, index) => sourceMp4.readUInt32BE(stsz + 20 + index * 4),
  );
  const stss = boxStart("stss");
  const syncCount = sourceMp4.readUInt32BE(stss + 12);
  const stco = boxStart("stco");
  const mdat = boxStart("mdat");
  const avc1 = boxStart("avc1", boxStart("stsd") + 8);
  const avcC = boxStart("avcC");

  assert.equal(sourceMp4.length, 10_979);
  assert.equal(sha256(sourceMp4), SOURCE_SHA256);
  assert.ok(boxStart("moov") < mdat);
  assert.equal(mdat + sourceMp4.readUInt32BE(mdat), sourceMp4.length);
  assert.equal(count, 30);
  assert.ok(new Set(sizes).size >= 12);
  assert.ok(syncCount >= 3);
  assert.equal(sourceMp4.readUInt32BE(stco + 12), 1);
  assert.equal(sourceMp4.readUInt32BE(stco + 16), mdat + 8);
  assert.equal(sourceMp4.readUInt16BE(avc1 + 32), 320);
  assert.equal(sourceMp4.readUInt16BE(avc1 + 34), 180);
  assert.deepEqual(sourceMp4.subarray(avcC + 9, avcC + 12), Buffer.from([0x42, 0xc0, 0x1e]));
  assert.deepEqual(sourceMp4.subarray(avcC + 16, avcC + 20), Buffer.from([0x67, 0x42, 0xc0, 0x1e]));
});

function boxStart(type, after = 0) {
  const typeOffset = sourceMp4.indexOf(Buffer.from(type), after);
  assert.ok(typeOffset >= 4, `missing ${type} box`);
  return typeOffset - 4;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}
