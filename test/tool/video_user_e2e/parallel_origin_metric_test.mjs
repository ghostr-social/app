import assert from "node:assert/strict";
import test from "node:test";
import {
  peakParallelOriginVideos,
} from "../../../tool/video_user_e2e/parallel_origin_metrics.mjs";

test("parallel origin evidence counts distinct simultaneously retrieved videos", () => {
  const requests = [
    request("v0", 0, 100),
    request("v0", 10, 90),
    request("v1", 20, 80),
    request("v2", 100, 150),
  ];

  assert.equal(peakParallelOriginVideos(requests), 2);
});

function request(video, started_at_ms, closed_at_ms) {
  return {video, method: "GET", started_at_ms, closed_at_ms};
}
