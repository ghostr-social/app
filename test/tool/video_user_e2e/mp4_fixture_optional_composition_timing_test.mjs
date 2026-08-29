import assert from "node:assert/strict";
import test from "node:test";
import {scaleAvcTiming} from "../../../tool/video_user_e2e/mp4_fixture_expansion.mjs";
import {playableMp4} from "../../../tool/video_user_e2e/media_fixture.mjs";

test("AVC timing scaling accepts streams without composition offsets", () => {
  const fixture = Buffer.from(playableMp4);
  const composition = fixture.indexOf(Buffer.from("ctts"));
  if (composition > 0) fixture.write("free", composition, "ascii");

  assert.doesNotThrow(() => scaleAvcTiming(fixture, 2));
});
