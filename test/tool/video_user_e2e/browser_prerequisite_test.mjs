import assert from "node:assert/strict";
import {chmod, mkdtemp, writeFile} from "node:fs/promises";
import {tmpdir} from "node:os";
import {join} from "node:path";
import test from "node:test";
import {verifyBrowser} from "../../../tool/video_user_e2e/prerequisites.mjs";

test("browser prerequisite accepts the configured executable without a local lock", async () => {
  const browser = await fakeBrowser();
  await assert.rejects(verifyBrowser(input(null)), /VIDEO_USER_E2E_BROWSER/);
  assert.deepEqual(await verifyBrowser(input(browser)), browser);
});

async function fakeBrowser() {
  const directory = await mkdtemp(join(tmpdir(), "ghostr-e2e-browser-"));
  const path = join(directory, "browser");
  const version = "Server Chromium 123.45";
  const body = `#!/bin/sh\nprintf '%s\\n' '${version}'\n`;
  await writeFile(path, body);
  await chmod(path, 0o755);
  return {path, version};
}

function input(browser) {
  return {
    environment: {VIDEO_USER_E2E_BROWSER: browser?.path},
  };
}
