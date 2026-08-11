import assert from "node:assert/strict";
import test from "node:test";
import {scaleAvcTiming} from "../../../tool/video_user_e2e/mp4_fixture_expansion.mjs";
import {playableMp4} from "../../../tool/video_user_e2e/media_fixture.mjs";

test("AVC timing scaling doubles the complete timeline without mutating its input", () => {
  const original = Buffer.from(playableMp4);
  const before = timingOf(playableMp4);
  const scaled = scaleAvcTiming(playableMp4, 2);

  assert.deepEqual(timingOf(scaled), scaledTiming(before, 2));
  assert.deepEqual(playableMp4, original);
  assert.throws(() => scaleAvcTiming(playableMp4, 0), /positive integer/);
  assert.throws(() => scaleAvcTiming(Buffer.alloc(8), 2), /missing mvhd/);
});

function timingOf(bytes) {
  return {
    movie: scalar(bytes, "mvhd", 24),
    track: scalar(bytes, "tkhd", 28),
    editDuration: scalar(bytes, "elst", 16),
    editStart: scalar(bytes, "elst", 20, true),
    media: scalar(bytes, "mdhd", 24),
    decoding: table(bytes, "stts"),
    composition: table(bytes, "ctts"),
  };
}

function scaledTiming(input, multiplier) {
  return {
    movie: input.movie * multiplier,
    track: input.track * multiplier,
    editDuration: input.editDuration * multiplier,
    editStart: input.editStart * multiplier,
    media: input.media * multiplier,
    decoding: scaleEntries(input.decoding, multiplier),
    composition: scaleEntries(input.composition, multiplier),
  };
}

function scaleEntries(entries, multiplier) {
  return entries.map(({count, value}) => ({count, value: value * multiplier}));
}

function scalar(bytes, type, offset, signed = false) {
  const start = boxStart(bytes, type);
  return signed ? bytes.readInt32BE(start + offset) : bytes.readUInt32BE(start + offset);
}

function table(bytes, type) {
  const start = boxStart(bytes, type);
  const count = bytes.readUInt32BE(start + 12);
  return Array.from({length: count}, (_, index) => ({
    count: bytes.readUInt32BE(start + 16 + index * 8),
    value: bytes.readUInt32BE(start + 20 + index * 8),
  }));
}

function boxStart(bytes, type) {
  return bytes.indexOf(Buffer.from(type)) - 4;
}
