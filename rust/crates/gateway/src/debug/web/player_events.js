const debugPlayer = byId("player");
let presentedFrame = false;
let presentedFrameCallback = null;
let presentationAuthority = 0;

function beginPresentationAttempt() {
  presentationAuthority += 1;
  presentedFrame = false;
  cancelPresentedFrameCallback();
}

function resetPresentedFrame() {
  beginPresentationAttempt();
  requestPresentedFrame(presentationAuthority, currentId);
}

function cancelPresentedFrameCallback() {
  if (presentedFrameCallback === null) return;
  debugPlayer.cancelVideoFrameCallback?.(presentedFrameCallback);
  presentedFrameCallback = null;
}

function requestPresentedFrame(authority, id) {
  if (presentedFrame || presentedFrameCallback !== null) return;
  if (typeof debugPlayer.requestVideoFrameCallback !== "function") return;
  presentedFrameCallback = debugPlayer.requestVideoFrameCallback(() => {
    presentedFrameCallback = null;
    if (authority !== presentationAuthority || id !== currentId) return;
    presentedFrame = true;
  });
}

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

debugPlayer.addEventListener("loadstart", () => {
  resetPresentedFrame();
  showPlaybackPhase("Loading");
});
debugPlayer.addEventListener("waiting", () => showPlaybackPhase("Buffering"));
debugPlayer.addEventListener("stalled", () => showPlaybackPhase("Stalled"));
debugPlayer.addEventListener("playing", () => {
  showPlaybackPhase("Playing");
  requestPresentedFrame(presentationAuthority, currentId);
});
debugPlayer.addEventListener("pause", () => showPlaybackPhase("Paused"));
debugPlayer.addEventListener("error", showPlaybackFailure);
debugPlayer.addEventListener("ended", playNext);
