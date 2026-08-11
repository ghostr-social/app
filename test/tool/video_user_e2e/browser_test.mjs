import assert from "node:assert/strict";
import test from "node:test";
import {
  browserArguments, parseDevToolsUrl,
} from "../../../tool/video_user_e2e/browser.mjs";

test("browser launch isolates profile, cache, and background work", () => {
  const arguments_ = browserArguments({profile: "/tmp/profile", cache: "/tmp/cache"});
  assert(arguments_.includes("--user-data-dir=/tmp/profile"));
  assert(arguments_.includes("--disk-cache-dir=/tmp/cache"));
  assert(arguments_.includes("--disable-background-networking"));
  assert(arguments_.includes("--disable-features=ServiceWorker"));
});

test("only an ephemeral loopback DevTools endpoint is accepted", () => {
  const local = "DevTools listening on ws://127.0.0.1:43123/devtools/browser/id";
  assert.equal(parseDevToolsUrl(local), "ws://127.0.0.1:43123/devtools/browser/id");
  assert.throws(
    () => parseDevToolsUrl("DevTools listening on ws://example.com:99/devtools/browser/id"),
    /loopback/,
  );
});
