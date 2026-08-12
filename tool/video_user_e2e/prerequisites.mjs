import {execFile} from "node:child_process";
import {access, constants, stat} from "node:fs";
import {isAbsolute} from "node:path";

export async function verifyBrowser(input) {
  const path = input.environment.VIDEO_USER_E2E_BROWSER;
  requireAbsolutePath(path);
  await requireExecutable(path);
  const version = await browserVersion(path);
  return {path, version};
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

function browserVersion(path) {
  return new Promise((resolve, reject) => {
    execFile(path, ["--version"], (error, stdout) => {
      if (error) reject(new Error(`cannot inspect browser executable: ${error.message}`));
      else resolve(stdout.trim());
    });
  });
}
