import {spawn} from "node:child_process";
import {TextRing} from "./artifacts.mjs";
import {trackSpawned} from "./owned_child.mjs";
import {waitForParsedOutput} from "./process_output.mjs";
import {localOnlyEnvironment, parseDashboardUrl} from "./server.mjs";

export async function startServer(input) {
  const log = new TextRing();
  const child = spawn("make", ["web"], {
    cwd: input.files.root,
    detached: true,
    stdio: ["ignore", "pipe", "pipe"],
    env: serverEnvironment(input),
  });
  const output = waitForParsedOutput({
    child,
    log,
    marker: "Video debug dashboard:",
    parse: parseDashboardUrl,
    timeoutMs: input.timeoutMs,
    label: "video debug server",
    signal: input.signal,
  });
  output.catch(() => {});
  await trackSpawned(child, input.lifecycle, "video debug server");
  return {child, log, url: await output};
}

function serverEnvironment({environment, files}) {
  return localOnlyEnvironment({
    ...environment,
    WEB_DEBUG_CACHE_DIR: files.mediaCache,
    WEB_DEBUG_STATE_DIR: files.serverState,
    WEB_DEBUG_RUST_DIR: files.rust,
  });
}
