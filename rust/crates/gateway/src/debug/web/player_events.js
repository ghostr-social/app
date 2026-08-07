const debugPlayer = byId("player");

function showPlaybackFailure(rejection) {
  const media = debugPlayer.error;
  const detail = media?.message || rejection?.message || "Browser rejected this media";
  const status = byId("player-state");
  updatePlaybackPhase("Error");
  status.textContent = media ? `Error ${media.code}: ${detail}` : `Blocked: ${detail}`;
  status.classList.add("player-error");
}

function showPlaybackPhase(phase) {
  updatePlaybackPhase(phase);
  byId("player-state").textContent = phase;
  byId("player-state").classList.remove("player-error");
}

debugPlayer.addEventListener("loadstart", () => showPlaybackPhase("Loading"));
debugPlayer.addEventListener("waiting", () => showPlaybackPhase("Buffering"));
debugPlayer.addEventListener("stalled", () => showPlaybackPhase("Stalled"));
debugPlayer.addEventListener("playing", () => showPlaybackPhase("Playing"));
debugPlayer.addEventListener("pause", () => showPlaybackPhase("Paused"));
debugPlayer.addEventListener("error", showPlaybackFailure);
debugPlayer.addEventListener("ended", playNext);
