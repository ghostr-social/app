import {attachRequestLedger} from "./request_ledger.mjs";
import {evaluate} from "./page_runtime.mjs";
import {poll} from "./wait.mjs";

export async function openPage({cdp, url, ledger, signal}) {
  const {targetId} = await cdp.send("Target.createTarget", {url: "about:blank"});
  const {sessionId} = await cdp.send("Target.attachToTarget", {targetId, flatten: true});
  const page = {cdp, sessionId, targetId};
  attachRequestLedger(cdp, ledger);
  await enablePage(page);
  await cdp.send("Page.navigate", {url}, sessionId);
  await poll({
    read: () => evaluate(page, "document.readyState"),
    accept: (state) => state === "complete",
    timeoutMs: 30_000,
    intervalMs: 100,
    label: "debug page load",
    signal,
  });
  return page;
}

async function enablePage(page) {
  await Promise.all([
    page.cdp.send("Page.enable", {}, page.sessionId),
    page.cdp.send("Runtime.enable", {}, page.sessionId),
    page.cdp.send("Network.enable", {}, page.sessionId),
  ]);
  await page.cdp.send("Network.clearBrowserCache", {}, page.sessionId);
  await page.cdp.send("Network.setCacheDisabled", {cacheDisabled: true}, page.sessionId);
  await page.cdp.send("Network.setBypassServiceWorker", {bypass: true}, page.sessionId);
}
