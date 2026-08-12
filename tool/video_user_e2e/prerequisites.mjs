import {createHash} from "node:crypto";
import {execFile} from "node:child_process";
import {access, constants, createReadStream, stat} from "node:fs";
import {isAbsolute} from "node:path";

export async function verifyBrowser(input) {
  const path = input.environment.VIDEO_USER_E2E_BROWSER;
  requireAbsolutePath(path);
  await requireExecutable(path);
  const expected = lockedBrowser(input);
  const version = await browserVersion(path);
  if (version !== expected.version) throw new Error(`browser version mismatch: ${version}`);
  const sha256 = await fileSha256(path);
  if (sha256 !== expected.sha256) throw new Error(`browser SHA-256 mismatch: ${sha256}`);
  return {path, version, sha256};
}

function requireAbsolutePath(path) {
  if (!path) throw new Error("VIDEO_USER_E2E_BROWSER is required");
  if (!isAbsolute(path)) throw new Error("VIDEO_USER_E2E_BROWSER must be absolute");
}

function requireExecutable(path) {
  return new Promise((resolve, reject) => {
    access(path, constants.X_OK, (error) => {
      if (error) return reject(new Error(`browser is missing or non-executable: ${path}`));
      stat(path, (issue, info) => {
        if (issue || !info.isFile()) {
          reject(new Error(`browser is not a regular executable: ${path}`));
        } else resolve();
      });
    });
  });
}

function lockedBrowser({lock, platform, architecture}) {
  const key = `${platform}-${architecture}`;
  const expected = lock[key];
  if (!expected) throw new Error(`no pinned browser for ${key}`);
  const valid = /^(Brave Browser|Google Chrome)$/.test(expected.product)
    && expected.version?.startsWith(expected.product)
    && /^[a-f0-9]{64}$/.test(expected.sha256 || "");
  if (!valid) throw new Error(`invalid pinned browser lock for ${key}`);
  return expected;
}

function browserVersion(path) {
  return new Promise((resolve, reject) => {
    execFile(path, ["--version"], (error, stdout) => {
      if (error) reject(new Error(`cannot inspect pinned browser: ${error.message}`));
      else resolve(stdout.trim());
    });
  });
}

function fileSha256(path) {
  return new Promise((resolve, reject) => {
    const digest = createHash("sha256");
    createReadStream(path)
      .on("error", reject)
      .on("data", (chunk) => digest.update(chunk))
      .on("end", () => resolve(digest.digest("hex")));
  });
}
