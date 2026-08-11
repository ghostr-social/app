import {readFile} from "node:fs/promises";
import {verifyBrowser} from "./prerequisites.mjs";

const LOCK = new URL("./browser_lock.json", import.meta.url);

export async function verifyPinnedBrowser(input = {}) {
  if (typeof globalThis.WebSocket !== "function") {
    throw new Error("Node.js with built-in WebSocket support is required");
  }
  const lock = JSON.parse(await readFile(LOCK, "utf8"));
  return verifyBrowser({
    environment: input.environment || process.env,
    platform: input.platform || process.platform,
    architecture: input.architecture || process.arch,
    lock,
  });
}
