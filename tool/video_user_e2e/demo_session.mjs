import {once} from "node:events";
import {OwnedLifecycle} from "./lifecycle.mjs";
import {startLocalOrigin} from "./local_origin.mjs";
import {registerOrderedVideos, selectVideoFocus} from "./ordered_admission.mjs";
import {createRunFiles, removeTransientRunFiles} from "./run_files.mjs";
import {startServer} from "./server_process.mjs";

const DEMO_NETWORK = Object.freeze({
  bandwidth_kbps: 900,
  latency_ms: 100,
  packet_loss_bps: 0,
  max_connections_per_host: 4,
});
const ORIGIN_OPTIONS = Object.freeze({
  virtualBytes: 8 * 1_024 * 1_024,
  chunkBytes: 32 * 1_024,
  chunkDelayMs: 120,
});

const DEFAULT_BOUNDARIES = Object.freeze({
  applyDemoNetwork,
  createLifecycle: () => new OwnedLifecycle(),
  createRunFiles,
  output: console.log,
  registerOrderedVideos,
  removeTransientRunFiles,
  selectVideoFocus,
  startLocalOrigin,
  startServer,
  waitForStop,
});

export function createVideoDemoRunner(overrides = {}) {
  const boundaries = {...DEFAULT_BOUNDARIES, ...overrides};
  return (input) => runDemoSession(boundaries, input);
}

export const runVideoDemo = createVideoDemoRunner();

async function runDemoSession(boundaries, {root, environment}) {
  const files = await boundaries.createRunFiles(root, environment);
  const lifecycle = boundaries.createLifecycle();
  let origin;
  try {
    origin = await boundaries.startLocalOrigin(ORIGIN_OPTIONS);
    const server = await startDemoServer(boundaries, files, lifecycle, environment);
    const ids = await admitDemo(boundaries, server.url, origin.url);
    announce(boundaries.output, server.url, ids.length);
    await boundaries.waitForStop();
  } finally {
    await lifecycle.teardown();
    await origin?.close();
    await boundaries.removeTransientRunFiles(files);
  }
}

async function startDemoServer(boundaries, files, lifecycle, environment) {
  return boundaries.startServer({
    files,
    lifecycle,
    environment,
    signal: new AbortController().signal,
    timeoutMs: 90_000,
  });
}

async function admitDemo(boundaries, server, origin) {
  const ids = await boundaries.registerOrderedVideos({
    server,
    origin,
    scenario: null,
    sizeBytes: ORIGIN_OPTIONS.virtualBytes,
  });
  await boundaries.applyDemoNetwork(server);
  await boundaries.selectVideoFocus(server, ids[0]);
  return ids;
}

function announce(output, server, count) {
  output(`WARP demo ready: ${server}`);
  output(`${count} videos admitted; watch up to 4 simultaneous retrieval lanes.`);
  output("Swipe with Next/Previous. Press Ctrl-C to stop.");
}

async function applyDemoNetwork(server, request = fetch) {
  const response = await request(`${server}/api/network`, {
    method: "PUT",
    headers: {"content-type": "application/json"},
    body: JSON.stringify(DEMO_NETWORK),
  });
  if (!response.ok) throw new Error(`demo network setup failed: ${response.status}`);
}

async function waitForStop(processState = process) {
  await Promise.race([once(processState, "SIGINT"), once(processState, "SIGTERM")]);
}
