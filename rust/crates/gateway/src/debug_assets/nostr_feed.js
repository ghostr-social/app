const FOCUS_ENDPOINT = "/debug/api/focus";

function relayRow(relay) {
  const row = document.createElement("div");
  row.className = `nostr-relay is-${relay.status}`;
  const status = document.createElement("i");
  const label = document.createElement("span");
  const value = document.createElement("b");
  label.textContent = relay.url;
  value.textContent = relay.status;
  row.append(status, label, value);
  return row;
}

function renderNostr(nostr) {
  byId("nostr-stage").textContent = nostr.stage;
  byId("nostr-count").textContent = nostr.discovered_count;
  const relays = byId("nostr-relays");
  relays.replaceChildren();
  nostr.relays.forEach((relay) => relays.append(relayRow(relay)));
  if (!nostr.relays.length) relays.innerHTML = '<p class="empty">No relays configured</p>';
}

async function selectNostrFocus(id) {
  const response = await fetch(FOCUS_ENDPOINT, {
    method: "PUT",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ id }),
  });
  if (!response.ok && response.status !== 404) {
    byId("updated").textContent = `Focus update failed: HTTP ${response.status}`;
  }
}
