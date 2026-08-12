import {spawn} from "node:child_process";
import {TextRing} from "./artifacts.mjs";
import {browserArguments, parseDevToolsUrl} from "./browser.mjs";
import {connectCdp} from "./cdp.mjs";
import {trackSpawned} from "./owned_child.mjs";
import {openPage} from "./page_session.mjs";
import {waitForParsedOutput} from "./process_output.mjs";
import {RequestLedger} from "./request_ledger.mjs";

export async function startBrowser(input) {
  const log = new TextRing();
  const child = spawn(input.browser.path, browserArguments({
    profile: input.files.profile,
    cache: input.files.browserCache,
  }), {
    cwd: input.files.root,
    detached: true,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const output = waitForParsedOutput({
    child,
    log,
    marker: "DevTools listening on",
    parse: parseDevToolsUrl,
    timeoutMs: input.timeoutMs,
    label: "browser",
    signal: input.signal,
  });
  output.catch(() => {});
  await trackSpawned(child, input.lifecycle, "browser");
  const cdp = await connectCdp(await output);
  const ledger = new RequestLedger();
  const page = await openPage({cdp, url: input.url, ledger, signal: input.signal});
  return {child, log, cdp, ledger, page};
}
