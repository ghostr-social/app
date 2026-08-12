export function localOnlyEnvironment(base) {
  return {
    ...base,
    GHOSTR_NOSTR_RELAYS: "",
    GHOSTR_NOSTR_SEARCH_RELAYS: "",
  };
}

export function parseDashboardUrl(output) {
  const match = output.match(/Video debug dashboard: (http:\/\/[^\s]+)/);
  if (!match) throw new Error("dashboard URL was not published");
  const url = new URL(match[1]);
  if (url.hostname !== "127.0.0.1") throw new Error("dashboard must bind loopback");
  if (!url.port) throw new Error("dashboard must use an ephemeral port");
  if (url.pathname !== "/debug") throw new Error("dashboard path must be /debug");
  return url.toString().replace(/\/$/, "");
}
