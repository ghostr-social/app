import {createServer} from "node:http";
import {once} from "node:events";
import {playableMp4} from "./media_fixture.mjs";
import {delay} from "./wait.mjs";

export async function startLocalOrigin(options = {}) {
  const settings = {
    virtualBytes: options.virtualBytes ?? 4 * 1024 * 1024,
    chunkBytes: options.chunkBytes ?? 64 * 1024,
    chunkDelayMs: options.chunkDelayMs ?? 20,
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
    close: () => close(server, active),
  };
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
  const id = new URL(request.url, "http://127.0.0.1").pathname
    .split("/").at(-1).replace(/\.mp4$/, "");
  requests.push({id, ...range, started_at_ms: Date.now()});
  writeHeaders(response, range, settings.virtualBytes, request.method);
  if (request.method === "HEAD") return response.end();
  for (let offset = range.start; offset < range.end; offset += settings.chunkBytes) {
    const end = Math.min(offset + settings.chunkBytes, range.end);
    if (!await writeChunk(response, mediaSlice(offset, end))) return;
    if (end < range.end) await delay(settings.chunkDelayMs);
  }
  if (!response.destroyed) response.end();
}

function writeChunk(response, bytes) {
  if (response.destroyed) return false;
  if (response.write(bytes)) return true;
  return new Promise((resolve) => {
    const done = (writable) => {
      response.off("drain", drained);
      response.off("close", closed);
      resolve(writable);
    };
    const drained = () => done(true);
    const closed = () => done(false);
    response.once("drain", drained);
    response.once("close", closed);
  });
}

function requestedRange(header, total) {
  if (!header) return {start: 0, end: total, partial: false};
  const match = /^bytes=(\d+)-(\d*)$/.exec(header);
  if (!match) return null;
  const start = Number(match[1]);
  const inclusive = match[2] ? Number(match[2]) : total - 1;
  if (!Number.isSafeInteger(start) || start >= total || inclusive < start) return null;
  return {start, end: Math.min(inclusive + 1, total), partial: true};
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
  const fixtureStart = Math.min(start, playableMp4.length);
  const fixtureEnd = Math.min(end, playableMp4.length);
  if (fixtureEnd > fixtureStart) {
    playableMp4.copy(bytes, fixtureStart - start, fixtureStart, fixtureEnd);
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
