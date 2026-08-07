const HLS_SESSION_ENDPOINT = "/debug/api/hls";
const HLS_MIME = "application/vnd.apple.mpegurl";
let activeHls = null;
let activeHlsSession = null;
let playbackGeneration = 0;

function debugVideos(state) {
  const progressive = state.videos.map((video) => ({ ...video, delivery: "progressive" }));
  const hls = (state.hls_videos || []).map((video) => ({
    ...video,
    ranges: [],
    downloaded_bytes: 0,
    downloaded_duration_ms: null,
    progress: null,
    complete: false,
    playback_url: null,
  }));
  return progressive.concat(hls);
}

async function startPlayback(video, player) {
  const generation = ++playbackGeneration;
  await releaseHlsPlayback();
  if (generation !== playbackGeneration) return;
  if (video.delivery === "hls") {
    await startHlsPlayback(video.id, player, generation);
    return;
  }
  player.src = video.playback_url;
  await player.play();
}

async function startHlsPlayback(id, player, generation) {
  const response = await fetch(HLS_SESSION_ENDPOINT, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ id }),
  });
  if (!response.ok) throw new Error(`HLS session failed: HTTP ${response.status}`);
  const session = await response.json();
  if (generation !== playbackGeneration) {
    await releaseSession(session.session_id);
    return;
  }
  activeHlsSession = session.session_id;
  playHlsSource(session.playback_url, player);
}

function playHlsSource(url, player) {
  if (player.canPlayType(HLS_MIME)) {
    player.src = url;
    player.play().catch(showPlaybackFailure);
    return;
  }
  if (typeof Hls === "undefined" || !Hls.isSupported()) {
    throw new Error("This browser cannot play HLS");
  }
  activeHls = new Hls();
  activeHls.on(Hls.Events.ERROR, (_, data) => handleHlsError(data));
  activeHls.on(Hls.Events.MANIFEST_PARSED, () => player.play().catch(showPlaybackFailure));
  activeHls.loadSource(url);
  activeHls.attachMedia(player);
}

function handleHlsError(data) {
  if (data.fatal) showPlaybackFailure(new Error(`HLS ${data.details}`));
}

async function releaseHlsPlayback() {
  if (activeHls) activeHls.destroy();
  activeHls = null;
  const session = activeHlsSession;
  activeHlsSession = null;
  if (session) await releaseSession(session);
}

async function releaseSession(session) {
  await fetch(`${HLS_SESSION_ENDPOINT}/${session}`, { method: "DELETE" });
}

window.addEventListener("pagehide", () => {
  if (activeHls) activeHls.destroy();
  if (!activeHlsSession) return;
  fetch(`${HLS_SESSION_ENDPOINT}/${activeHlsSession}`, {
    method: "DELETE",
    keepalive: true,
  });
});
