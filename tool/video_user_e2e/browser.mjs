export function browserArguments({profile, cache}) {
  return [
    "--headless=new",
    "--remote-debugging-port=0",
    `--user-data-dir=${profile}`,
    `--disk-cache-dir=${cache}`,
    "--disable-application-cache",
    "--disable-background-networking",
    "--disable-component-update",
    "--disable-features=ServiceWorker",
    "--no-default-browser-check",
    "--no-first-run",
    "--use-mock-keychain",
    "--mute-audio",
    "about:blank",
  ];
}

export function parseDevToolsUrl(output) {
  const match = output.match(/DevTools listening on (ws:\/\/[^\s]+)/);
  if (!match) throw new Error("browser did not publish a DevTools URL");
  const url = new URL(match[1]);
  if (url.hostname !== "127.0.0.1" && url.hostname !== "localhost") {
    throw new Error("DevTools endpoint must be loopback");
  }
  if (!url.port) throw new Error("DevTools endpoint must use an ephemeral port");
  return url.toString();
}
