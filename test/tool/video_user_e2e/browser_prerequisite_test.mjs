import assert from "node:assert/strict";
import {createHash} from "node:crypto";
import {chmod, mkdtemp, writeFile} from "node:fs/promises";
import {tmpdir} from "node:os";
import {join} from "node:path";
import test from "node:test";
import {verifyBrowser} from "../../../tool/video_user_e2e/prerequisites.mjs";

test("browser prerequisite accepts only the exact locked executable", async () => {
  const browser = await fakeBrowser();
  await assert.rejects(verifyBrowser(input(null)), /VIDEO_USER_E2E_BROWSER/);
  await assert.rejects(
    verifyBrowser(input(browser, {expectedVersion: "Brave Browser 9"})),
    /version mismatch/,
  );
  await assert.rejects(
    verifyBrowser(input(browser, {sha256: "0".repeat(64)})),
    /SHA-256 mismatch/,
  );
  assert.deepEqual(await verifyBrowser(input(browser)), browser);
});

async function fakeBrowser() {
  const directory = await mkdtemp(join(tmpdir(), "ghostr-e2e-browser-"));
  const path = join(directory, "browser");
  const version = "Brave Browser 1.2.3";
  const body = `#!/bin/sh\nprintf '%s\\n' '${version}'\n`;
  await writeFile(path, body);
  await chmod(path, 0o755);
  return {path, version, sha256: createHash("sha256").update(body).digest("hex")};
}

function input(browser, override = {}) {
  const version = override.expectedVersion || browser?.version;
  const sha256 = override.sha256 || browser?.sha256;
  return {
    environment: {VIDEO_USER_E2E_BROWSER: browser?.path},
    platform: "test",
    architecture: "arch",
    lock: {"test-arch": {product: "Brave Browser", version, sha256}},
  };
}
