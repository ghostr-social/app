import {createServer} from "node:http";
import {once} from "node:events";
import {playableMedia} from "./media_fixture.mjs";
import {
  commitOriginFailure, createOriginFailurePlan, planOriginFailure,
} from "./origin_failure_plan.mjs";
import {requestedRange} from "./requested_range.mjs";
import {writeResponseChunk} from "./response_chunk.mjs";
import {delay} from "./wait.mjs";

export async function startLocalOrigin(options = {}) {
  const settings = {
    virtualBytes: options.virtualBytes ?? playableMedia.bytes.length,
    chunkBytes: options.chunkBytes ?? 64 * 1024,
    chunkDelayMs: options.chunkDelayMs ?? 20,
    failurePlan: createOriginFailurePlan(options),
    abortAfterBytes: options.abortAfterBytes ?? Number.POSITIVE_INFINITY,
    failSource: options.failSource ?? null,
    nextEventOrdinal: 0,
  };
  const requests = [];
  const active = new Set();
  const server = createServer((request, response) => {
    trackRequest(active, serve(request, response, settings, requests), response);
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address();
  return {
    url: `http://127.0.0.1:${address.port}`,
    requests,
    activeRequests: () => active.size,
    waitForIdle: () => waitForIdle(active),
    close: () => close(server, active),
  };
}

async function waitForIdle(active) {
  while (active.size > 0) {
    await Promise.allSettled([...active]);
  }
}

function trackRequest(active, serving, response) {
  const tracked = serving
    .catch((error) => response.destroy(error))
    .finally(() => active.delete(tracked));
  active.add(tracked);
}

async function serve(request, response, settings, requests) {
  const range = requestedRange(request.headers.range, settings.virtualBytes);
  if (!range) return rejectRange(response, settings.virtualBytes);
  const id = requestId(request.url);
  const outcome = requestOutcome({
    id, range, requestOrdinal: requests.length + 1, settings, method: request.method,
  });
  requests.push(outcome);
  if (id.includes(settings.failSource ?? "\0")) return failSource(response, outcome);
  writeHeaders(response, range, settings.virtualBytes, request.method);
  await serveBody(request.method, {response, range, settings, outcome});
}

function requestId(url) {
  return new URL(url, "http://127.0.0.1").pathname
    .split("/").at(-1).replace(/\.mp4$/, "");
}

async function serveBody(method, transfer) {
  if (method === "HEAD") return completeResponse(transfer.response, transfer.outcome);
  await writeBody(transfer);
}

async function writeBody({response, range, settings, outcome}) {
  for (let offset = range.start; offset < range.end; offset += settings.chunkBytes) {
    const end = Math.min(offset + settings.chunkBytes, range.end);
    const bytes = mediaSlice(offset, end);
    const written = writeResponseChunk(
      response, bytes, () => recordChunk(settings, outcome, bytes.length),
    );
    if (!await written) return finalize(outcome, false);
    if (shouldAbort(outcome, settings)) return abortResponse(response, outcome);
    if (end < range.end) await delay(settings.chunkDelayMs);
  }
  await completeResponse(response, outcome);
}

function recordChunk(settings, outcome, bytes) {
  outcome.bytes_sent += bytes;
  outcome.chunk_events.push({at_ms: Date.now(), ordinal: takeEventOrdinal(settings), bytes});
}

function requestOutcome(input) {
  const {id, range, requestOrdinal, settings, method} = input;
  const video = id.split("-")[0];
  const failure = planOriginFailure(settings.failurePlan, {video, method, requestOrdinal});
  return {
    id,
    video,
    method,
    ...range,
    started_at_ms: Date.now(),
    bytes_sent: 0,
    chunk_events: [],
    start_ordinal: method === "HEAD" ? null : takeEventOrdinal(settings),
    completed: false,
    canceled: false,
    ...failure,
    planned_failure: failure.targeted_failure || failure.periodic_failure,
    injected_failure: false,
  };
}

function takeEventOrdinal(settings) {
  const ordinal = settings.nextEventOrdinal;
  settings.nextEventOrdinal += 1;
  return ordinal;
}

function shouldAbort(outcome, settings) {
  if (!outcome.planned_failure || outcome.bytes_sent < settings.abortAfterBytes) return false;
  return commitOriginFailure(settings.failurePlan, outcome);
}

function abortResponse(response, outcome) {
  outcome.injected_failure = true;
  response.destroy();
  finalize(outcome, false);
}

async function failSource(response, outcome) {
  outcome.failed_status = 503;
  response.writeHead(503);
  await completeResponse(response, outcome);
}

async function completeResponse(response, outcome) {
  const completed = await endResponse(response);
  finalize(outcome, completed);
}

function endResponse(response) {
  if (response.destroyed) return false;
  return new Promise((resolve) => {
    const finish = () => done(true);
    const close = () => done(false);
    const done = (completed) => {
      response.off("finish", finish);
      response.off("close", close);
      resolve(completed);
    };
    response.once("finish", finish);
    response.once("close", close);
    response.end();
  });
}

function finalize(outcome, completed) {
  outcome.completed = completed;
  outcome.canceled = !completed && !outcome.injected_failure;
  outcome.closed_at_ms = Date.now();
}

function writeHeaders(response, range, total, method) {
  response.statusCode = range.partial ? 206 : 200;
  response.setHeader("accept-ranges", "bytes");
  response.setHeader("content-type", "video/mp4");
  response.setHeader("content-length", range.end - range.start);
  if (range.partial) {
    response.setHeader("content-range", `bytes ${range.start}-${range.end - 1}/${total}`);
  }
  if (method === "HEAD") response.statusCode = 200;
}

function rejectRange(response, total) {
  response.writeHead(416, {"content-range": `bytes */${total}`});
  response.end();
}

function mediaSlice(start, end) {
  const bytes = Buffer.alloc(end - start);
  const fixtureStart = Math.min(start, playableMedia.bytes.length);
  const fixtureEnd = Math.min(end, playableMedia.bytes.length);
  if (fixtureEnd > fixtureStart) {
    playableMedia.bytes.copy(bytes, fixtureStart - start, fixtureStart, fixtureEnd);
  }
  return bytes;
}

async function close(server, active) {
  const closed = new Promise((resolve, reject) => {
    server.close((error) => error ? reject(error) : resolve());
    server.closeAllConnections?.();
  });
  await closed;
  await Promise.allSettled([...active]);
}
