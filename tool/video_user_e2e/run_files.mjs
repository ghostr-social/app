import {mkdir, mkdtemp, rm} from "node:fs/promises";
import {isAbsolute, join, relative, resolve, sep} from "node:path";

export async function createRunFiles(root, environment = {}) {
  const transient = resolve(root, "rust/target/video-user-e2e");
  const retained = retainedRoot(root, environment.VIDEO_USER_E2E_ARTIFACT_ROOT);
  await Promise.all([
    mkdir(transient, {recursive: true}),
    mkdir(retained, {recursive: true}),
  ]);
  const run = await mkdtemp(join(transient, "run-"));
  const artifacts = await mkdtemp(join(retained, "run-"));
  return {
    root: resolve(root),
    run,
    artifacts,
    profile: join(run, "browser-profile"),
    browserCache: join(run, "browser-cache"),
    mediaCache: join(run, "video-debug-cache"),
    serverState: join(run, "server-state"),
    rust: resolve(root, "rust"),
  };
}

function retainedRoot(root, configured) {
  if (configured && !isAbsolute(configured)) {
    throw new Error("VIDEO_USER_E2E_ARTIFACT_ROOT must be absolute");
  }
  const retained = resolve(configured || resolve(root, ".artifacts/video-user-e2e"));
  if (inside(resolve(root, "rust/target"), retained)) {
    throw new Error("video E2E artifacts must be outside Cargo target");
  }
  return retained;
}

export async function removeTransientRunFiles(files) {
  for (const path of transientPaths(files)) {
    requireInside(files.run, path);
    await rm(path, {recursive: true, force: true});
  }
}

function transientPaths(files) {
  return [files.profile, files.browserCache, files.mediaCache, files.serverState];
}

function requireInside(root, path) {
  const prefix = `${resolve(root)}${sep}`;
  if (!resolve(path).startsWith(prefix)) {
    throw new Error(`unsafe E2E cleanup path: ${path}`);
  }
}

function inside(parent, child) {
  const path = relative(parent, child);
  return path === "" || (!path.startsWith("..") && !isAbsolute(path));
}
