import {captureScreenshot} from "./page_runtime.mjs";

export async function writeSuccess(context, trace) {
  await writeCommon(context, trace);
  await context.store.writeJson("result.json", {
    outcome: "passed",
    completed_at: new Date().toISOString(),
  });
}

export async function writeFailure(context, error) {
  await writeCommon(context, context.trace || null);
  await context.store.writeJson("failure.json", {
    name: error?.name || "Error",
    message: error?.message || String(error),
    stack: error?.stack,
    failed_at: new Date().toISOString(),
  });
  await failureScreenshot(context);
}

async function writeCommon(context, trace) {
  await Promise.all([
    context.store.writeJson("trace.json", trace),
    context.store.writeJson("browser-requests.json", context.browserRun?.ledger.entries || []),
    context.store.writeJson("origin-requests.json", context.origin?.requests || []),
    context.store.writeText("server.log", context.server?.log.toString() || ""),
    context.store.writeText("browser.log", context.browserRun?.log.toString() || ""),
  ]);
}

async function failureScreenshot(context) {
  if (!context.browserRun?.page) return;
  try {
    const image = await captureScreenshot(context.browserRun.page);
    await context.store.writeBase64("failure.png", image);
  } catch (error) {
    await context.store.writeJson("screenshot-error.json", {message: error.message});
  }
}
