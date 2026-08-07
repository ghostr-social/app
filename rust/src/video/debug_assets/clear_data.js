const clearDebugButton = byId("clear-debug-data");

async function stopDebugPlayback() {
  playbackGeneration += 1;
  await releaseHlsPlayback();
  const player = byId("player");
  player.pause();
  player.removeAttribute("src");
  player.load();
}

function resetBrowserState() {
  previous.clear();
  rates.clear();
  playedIds.clear();
  currentId = null;
  latestState = null;
  playbackPhase = "Idle";
}

async function clearDebugData() {
  if (!window.confirm("Delete every discovered event and downloaded video?")) return;
  clearDebugButton.disabled = true;
  clearDebugButton.textContent = "Clearing…";
  try {
    await stopDebugPlayback();
    const response = await fetch("/debug/api/data", { method: "DELETE" });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    resetBrowserState();
    await refresh();
  } catch (error) {
    byId("updated").textContent = `Clear failed: ${error.message}`;
  } finally {
    clearDebugButton.disabled = false;
    clearDebugButton.textContent = "Clear debug data";
  }
}

clearDebugButton.addEventListener("click", clearDebugData);
