import assert from "node:assert/strict";
import {createHash} from "node:crypto";
import test from "node:test";
import {sourceMp4} from "../../../tool/video_user_e2e/media_fixture.mjs";

const SOURCE_SHA256 =
  "f9f8f3ca660be3b6a7e8d9f8dd818a215e061a6e88c120dea919b0595453c5bc";

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
  const avcC = boxStart("avcC");

  assert.equal(sourceMp4.length, 2_632);
  assert.equal(sha256(sourceMp4), SOURCE_SHA256);
  assert.ok(boxStart("moov") < mdat);
  assert.equal(mdat + sourceMp4.readUInt32BE(mdat), sourceMp4.length);
  assert.equal(count, 30);
  assert.ok(new Set(sizes).size >= 12);
  assert.ok(syncCount >= 3);
  assert.equal(sourceMp4[boxStart("ctts") + 8], 0);
  assert.equal(sourceMp4.readUInt32BE(stco + 12), 1);
  assert.equal(sourceMp4.readUInt32BE(stco + 16), mdat + 8);
  assert.equal(sourceMp4[avcC + 9], 100);
  assert.equal(sourceMp4[avcC + 11], 10);
});

function boxStart(type) {
  const typeOffset = sourceMp4.indexOf(Buffer.from(type));
  assert.ok(typeOffset >= 4, `missing ${type} box`);
  return typeOffset - 4;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}
