import {ArtifactStore} from "./artifacts.mjs";
import {startBrowser} from "./browser_process.mjs";
import {validateJourney} from "./journey_outcome.mjs";
import {OwnedLifecycle} from "./lifecycle.mjs";
import {startLocalOrigin} from "./local_origin.mjs";
import {
  controlPoint, debugSnapshot, dispatchTrustedClick, playerSnapshot,
} from "./page_runtime.mjs";
import {createRunFiles, removeTransientRunFiles} from "./run_files.mjs";
import {startServer} from "./server_process.mjs";
import {writeFailure, writeSuccess} from "./trace_artifacts.mjs";
import {poll, withDeadline} from "./wait.mjs";

const TOTAL_TIMEOUT_MS = 180_000;
const VIRTUAL_BYTES = 4 * 1024 * 1024;

export async function runVideoUserE2e({root, environment, browser}) {
  const files = await createRunFiles(root, environment);
  const context = createContext(files, environment, browser);
  try {
    const trace = await withDeadline({
      run: (signal) => runScenario(context, signal),
      timeoutMs: TOTAL_TIMEOUT_MS,
      label: "local video user E2E",
    });
    context.trace = trace;
    await writeSuccess(context, trace);
    return {artifacts: files.artifacts, trace};
  } catch (error) {
    await writeFailure(context, error);
    throw new Error(`${error.message}; artifacts: ${files.artifacts}`, {cause: error});
  } finally {
    context.browserRun?.cdp.close();
    await context.origin?.close();
    await context.lifecycle.teardown();
    await removeTransientRunFiles(files);
  }
}

async function runScenario(context, signal) {
  context.origin = await startLocalOrigin();
  context.server = await startServer({...context, signal, timeoutMs: 90_000});
  const ids = await registerVideos(context.server.url, context.origin.url);
  context.browserRun = await startBrowser({
    ...context, signal, url: context.server.url, timeoutMs: 30_000,
  });
  await waitForVideos(context.browserRun.page, ids, signal);
  await requireUserStartsPlayback(context.browserRun.page);
  const trace = {clicks: [], samples: [], requests: []};
  const started = Date.now();
  for (const id of [ids[0], ids[1], ids[0]]) {
    await clickVideo(context.browserRun.page, id, signal);
    trace.clicks.push({id, at_ms: Date.now() - started});
    await watchProgress({
      page: context.browserRun.page, id, trace, started, signal,
    });
  }
  trace.requests = context.browserRun.ledger.entries;
  validateJourney(trace);
  return trace;
}

async function registerVideos(server, origin) {
  const ids = [];
  for (const name of ["a", "b"]) {
    const response = await fetch(`${server}/api/videos`, {
      method: "POST",
      headers: {"content-type": "application/json"},
      body: JSON.stringify({
        url: `${origin}/${name}.mp4`,
        size_bytes: VIRTUAL_BYTES,
        duration_ms: 3_000,
      }),
    });
    if (response.status !== 201) throw new Error(`video registration failed: ${response.status}`);
    ids.push((await response.json()).id);
  }
  return ids;
}

async function waitForVideos(page, ids, signal) {
  await poll({
    read: () => debugSnapshot(page),
    accept: (state) => ids.every((id) => state?.videos.some((video) => video.id === id)),
    timeoutMs: 15_000,
    intervalMs: 100,
    label: "local videos in debug state",
    signal,
  });
}

async function requireUserStartsPlayback(page) {
  const player = await playerSnapshot(page);
  if (player.id !== null || !player.paused) {
    throw new Error("playback started before a visible user control was clicked");
  }
}

async function clickVideo(page, id, signal) {
  const point = await poll({
    read: () => controlPoint(page, id),
    accept: (value) => value?.ready === true,
    timeoutMs: 10_000,
    intervalMs: 100,
    label: `${id} visible play control`,
    signal,
  });
  if (!point.label?.startsWith("Play ")) throw new Error("untrusted play control label");
  await dispatchTrustedClick(page, point);
}

async function watchProgress(input) {
  const first = await collectUntil({...input, accept: (sample) => {
    return sample.player.id === input.id && sample.player.phase === "playing";
  }, label: `${input.id} playing`});
  await collectUntil({...input, accept: (sample) => {
    return sample.player.id === input.id && sample.player.phase === "playing"
      && sample.player.current_time >= first.player.current_time + 0.5;
  }, label: `${input.id} media time`});
}

function collectUntil(input) {
  return poll({
    read: async () => {
      const sample = {
        at_ms: Date.now() - input.started,
        player: await playerSnapshot(input.page),
        state: await debugSnapshot(input.page),
      };
      input.trace.samples.push(sample);
      return sample;
    },
    accept: input.accept,
    timeoutMs: 15_000,
    intervalMs: 100,
    label: input.label,
    signal: input.signal,
  });
}

function createContext(files, environment, browser) {
  return {
    files,
    environment,
    browser,
    lifecycle: new OwnedLifecycle(),
    store: new ArtifactStore({directory: files.artifacts}),
    origin: null,
    server: null,
    browserRun: null,
    trace: null,
  };
}
