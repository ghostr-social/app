const networkModal = byId("network-modal");

function formProfile() {
  return {
    bandwidth_kbps: Number(byId("bandwidth").value),
    latency_ms: Number(byId("latency").value),
    max_connections_per_host: Number(byId("connections-limit").value),
  };
}

async function applyNetwork(event) {
  event.preventDefault();
  const button = event.currentTarget.querySelector('button[type="submit"]');
  button.disabled = true;
  byId("form-status").textContent = "Applying conditions…";
  try {
    const response = await fetch(NETWORK_ENDPOINT, {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(formProfile()),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    byId("form-status").textContent = "Conditions are active.";
    await refresh();
  } catch (error) {
    byId("form-status").textContent = `Could not apply: ${error.message}`;
  } finally {
    button.disabled = false;
  }
}

byId("network-button").addEventListener("click", () => networkModal.showModal());
byId("network-form").addEventListener("submit", applyNetwork);
networkModal.querySelector("[data-close-network]").addEventListener("click", () => networkModal.close());
networkModal.addEventListener("click", (event) => {
  if (event.target === networkModal) networkModal.close();
});
