export function successfulRunnerBoundaries() {
  const events = [];
  const ids = Array.from({length: 8}, (_, index) => `video-${index}`);
  const snapshots = [snapshot(ids, 0), snapshot(ids, 49_152)];
  const origin = {url: "http://127.0.0.1:4100", requests: [],
    close: async () => events.push("origin.close")};
  const browserRun = {
    page: {},
    cdp: {close: () => events.push("browser.close")},
    ledger: {entries: [mediaRequest()]},
  };
  let capturedTrace;
  const boundaries = {
    createRunFiles: async () => files(),
    delay: async (milliseconds) => events.push(`observe:${milliseconds}`),
    removeTransientRunFiles: async () => events.push("files.remove"),
    startLocalOrigin: async () => origin,
    startServer: async () => ({url: "http://127.0.0.1:4200"}),
    startBrowser: async () => browserRun,
    registerOrderedVideos: async () => ids,
    selectVideoFocus: async (_, id) => events.push(`focus:${id}`),
    refreshDebugSnapshot: async () => snapshots.shift() ?? snapshot(ids, 49_152),
    requireUserStartsPlayback: async () => events.push("playback.guard"),
    clickVideo: async (_, id) => events.push(`click:${id}`),
    watchProgress: async (input) => addProgress(input),
    sendControlAction: async () => events.push("control"),
    writeSuccess: async (_, trace) => { capturedTrace = trace; },
    writeFailure: async () => events.push("failure.write"),
  };
  return {boundaries, browserRun, events, ids, origin, trace: () => capturedTrace};
}

export function files() {
  return {root: "/tmp/video-runner", run: "/tmp/video-runner/run",
    artifacts: "/tmp/video-runner/artifacts", profile: "/tmp/video-runner/run/profile",
    browserCache: "/tmp/video-runner/run/browser-cache",
    mediaCache: "/tmp/video-runner/run/media-cache",
    serverState: "/tmp/video-runner/run/server-state", rust: "/tmp/video-runner/rust"};
}

function snapshot(ids, downloaded) {
  return {videos: ids.map((id) => ({
    id, downloaded_bytes: downloaded, total_bytes: 370_912,
  }))};
}

function mediaRequest() {
  return {url: "http://127.0.0.1:4200/video.mp4", method: "GET",
    range: "bytes=0-65535", status: 206,
    content_range: "bytes 0-65535/370912", finished: true};
}

async function addProgress(input) {
  const click = input.trace.clicks.at(-1);
  const ids = input.trace.ordered_video_ids;
  input.trace.samples.push(sample(ids, input.id, click.at_ms + 1, 0));
  input.trace.samples.push(sample(ids, input.id, click.at_ms + 751, 1));
  await new Promise((resolve) => setTimeout(resolve, 5));
}

function sample(ids, id, at_ms, current_time) {
  return {at_ms, player: {id, phase: "playing", current_time},
    state: {
      videos: ids.map((videoId) => ({id: videoId,
        downloaded_bytes: 65_536, total_bytes: 370_912})),
      network: {bandwidth_kbps: 2_500, latency_ms: 450, max_connections_per_host: 3},
      connections: [{host: "127.0.0.1", active: 1}],
    }};
}
