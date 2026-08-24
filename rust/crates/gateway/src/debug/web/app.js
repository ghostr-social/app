const byId = (id) => document.getElementById(id);
const NETWORK_ENDPOINT = "/debug/api/network";
const previous = new Map();
const rates = new Map();
const playedIds = new Set();
let currentId = null;
let latestState = null;
let profileLoaded = false;
let playbackPhase = "Idle";
function bytes(value) {
  if (value == null) return "Unknown";
  const units = ["B", "KB", "MB", "GB"];
  let amount = value;
  let unit = 0;
  while (amount >= 1000 && unit < units.length - 1) {
    amount /= 1000;
    unit += 1;
  }
  return `${amount.toFixed(unit ? 1 : 0)} ${units[unit]}`;
}
function duration(ms) {
  if (ms == null) return "Unknown";
  const seconds = Math.round(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  return `${minutes}m ${String(seconds % 60).padStart(2, "0")}s`;
}
function seconds(ms) { return ms == null ? "0s" : `${Math.floor(ms / 1000)}s`; }
function shortId(id) {
  return id.length > 24 ? `${id.slice(0, 12)}…${id.slice(-7)}` : id;
}
function observedRate(video, now) {
  const old = previous.get(video.id);
  const elapsed = old ? (now - old.at) / 1000 : 0;
  const fresh = elapsed > 0 ? Math.max(0, video.downloaded_bytes - old.bytes) / elapsed : 0;
  const smoothed = fresh > 0 ? fresh : (rates.get(video.id) || 0) * 0.65;
  previous.set(video.id, { bytes: video.downloaded_bytes, at: now });
  rates.set(video.id, smoothed < 1 ? 0 : smoothed);
  return smoothed;
}
function queueState(video) {
  if (video.id === currentId) return playbackPhase;
  return playedIds.has(video.id) ? "Played" : "Up next";
}
function queueRank(video) {
  if (video.id === currentId) return 0;
  if (video.focus_distance > 0) return video.focus_distance;
  if (video.focus_distance < 0) return 10_000 + Math.abs(video.focus_distance);
  return { "Up next": 1_000, Played: 20_000 }[queueState(video)];
}
function renderQueueRow(video, index) {
  const row = byId("video-row-template").content.firstElementChild.cloneNode(true);
  const state = queueState(video);
  row.dataset.videoId = video.id;
  row.classList.toggle("is-playing", video.id === currentId);
  row.classList.toggle("is-played", state === "Played");
  row.querySelector(".row-index").textContent = video.id === currentId ? (state === "Playing" ? "▶" : "…") : index + 1;
  row.querySelector("h3").textContent = video.title || shortId(video.id);
  row.querySelector(".queue-state").textContent = state;
  row.querySelector(".host").textContent = [video.creator, video.source_host].filter(Boolean).join(" · ") || "Unknown media host";
  row.querySelector("progress").value = video.progress || 0;
  row.querySelector(".buffered").textContent = `${seconds(video.downloaded_duration_ms)} ready`;
  row.querySelector(".bytes").textContent = `${Math.round((video.progress || 0) * 100)}%`;
  row.querySelector(".row-play").setAttribute("aria-label", `Play ${video.id}`);
  row.querySelector(".row-play").addEventListener("click", () => play(video));
  return row;
}
function renderQueue(videos) {
  const queue = byId("video-queue");
  queue.replaceChildren();
  const ordered = [...videos].sort((a, b) => queueRank(a) - queueRank(b));
  ordered.forEach((video, index) => queue.append(renderQueueRow(video, index)));
  if (!ordered.length) queue.innerHTML = '<p class="empty">Add a progressive video to begin.</p>';
  byId("video-count").textContent = `${videos.length} video${videos.length === 1 ? "" : "s"}`;
}
function selectedVideo(videos) {
  return videos.find((video) => video.id === currentId)
    || videos.find((video) => !video.complete && video.downloaded_bytes > 0)
    || videos[0];
}
function renderSelected(video) {
  const progress = video?.progress || 0;
  const rate = video ? rates.get(video.id) || 0 : 0;
  byId("inspector-title").textContent = video?.title || (video ? shortId(video.id) : "No video selected");
  byId("inspector-host").textContent = video?.source_host || "—";
  byId("buffered-seconds").textContent = seconds(video?.downloaded_duration_ms);
  byId("total-duration").textContent = duration(video?.duration_ms);
  byId("selected-progress").value = progress;
  byId("selected-percent").textContent = `${(progress * 100).toFixed(1)}%`;
  byId("selected-bytes").textContent = `${bytes(video?.downloaded_bytes || 0)} / ${bytes(video?.total_bytes)}`;
  byId("selected-ranges").textContent = video ? rangesText(video.ranges) : "No stored ranges";
  byId("inspector-status").textContent = rate > 1 ? "Downloading" : video?.status || "Queued";
}
function activityRow(video) {
  const row = document.createElement("div");
  const rate = rates.get(video.id) || 0;
  row.className = "activity-row";
  const label = document.createElement("span");
  const value = document.createElement("strong");
  label.textContent = `${video.title || shortId(video.id)} · ${rate > 1 ? `${bytes(rate)}/s` : video.status}`;
  value.textContent = seconds(video.downloaded_duration_ms);
  row.append(label, value);
  return row;
}
function renderActivity(videos) {
  const target = byId("download-activity");
  target.replaceChildren();
  videos.forEach((video) => target.append(activityRow(video)));
  if (!videos.length) target.innerHTML = '<p class="empty">No active downloads</p>';
}
function renderConnections(connections) {
  const target = byId("connections");
  target.replaceChildren();
  connections.forEach((item) => {
    const row = document.createElement("div");
    row.className = "connection";
    row.innerHTML = `<span></span><b>${item.active} active</b>`;
    row.querySelector("span").textContent = item.host;
    target.append(row);
  });
  if (!connections.length) target.innerHTML = '<p class="empty">No active connections</p>';
}
function renderMetrics(state, totalRate) {
  const active = state.connections.reduce((sum, item) => sum + item.active, 0);
  byId("stored").textContent = bytes(state.storage.used_bytes);
  byId("storage-total").textContent = `${bytes(state.storage.known_bytes)} planned`;
  byId("cached").textContent = state.storage.complete_count;
  byId("rate").textContent = totalRate > 1 ? `${bytes(totalRate)}/s` : "Idle";
  byId("relay-count").textContent = state.connections.length;
  byId("connection-count").textContent = `${active} connections`;
}
function renderPhone(video) {
  byId("player-title").textContent = video?.title || (video ? shortId(video.id) : "Nothing playing");
  byId("player-host").textContent = video?.source_host || "Waiting for the queue";
  byId("phone-buffered").textContent = video ? duration(video.downloaded_duration_ms) : "—";
  byId("phone-progress").value = video?.progress || 0;
  byId("player-empty").hidden = Boolean(video);
}
function loadProfile(profile) {
  if (profileLoaded) return;
  byId("bandwidth").value = profile.bandwidth_kbps;
  byId("latency").value = profile.latency_ms;
  byId("packet-loss").value = profile.packet_loss_bps;
  byId("connections-limit").value = profile.max_connections_per_host;
  profileLoaded = true;
}
function rangesText(items) {
  if (!items.length) return "No stored ranges";
  return items.map((item) => `${item.start.toLocaleString()}–${item.end.toLocaleString()}`).join(", ");
}
function render(state) {
  latestState = state;
  currentId ??= state.nostr.current_id;
  const videos = debugVideos(state);
  const now = performance.now();
  let totalRate = 0;
  state.videos.forEach((video) => { totalRate += observedRate(video, now); });
  const selected = selectedVideo(videos);
  renderQueue(videos);
  renderSelected(selected);
  renderPhone(videos.find((video) => video.id === currentId));
  renderDeliveryActivity(state);
  renderConnections(state.connections);
  renderMetrics(state, totalRate);
  renderReadyReserve(state.adaptive_plans);
  renderNostr(state.nostr);
  loadProfile(state.network);
  byId("updated").textContent = `Updated ${new Date().toLocaleTimeString()}`;
}
function updatePlaybackPhase(phase) { playbackPhase = phase; if (latestState) render(latestState); }
function play(video) {
  if (currentId && currentId !== video.id) playedIds.add(currentId);
  currentId = video.id;
  playbackPhase = "Loading";
  playedIds.delete(video.id);
  const player = byId("player");
  selectNostrFocus(video.id);
  startPlayback(video, player).catch(showPlaybackFailure);
  if (latestState) render(latestState);
}
function playNext() {
  if (!latestState || !currentId) return;
  playedIds.add(currentId);
  const videos = debugVideos(latestState);
  const index = videos.findIndex((video) => video.id === currentId);
  const next = videos.find((video) => video.focus_distance === 1) || videos[index + 1];
  currentId = null;
  next ? play(next) : render(latestState);
}
async function refresh() {
  try {
    const response = await fetch("/debug/api/state", { cache: "no-store" });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    render(await response.json());
  } catch (error) {
    byId("updated").textContent = `Unavailable: ${error.message}`;
  }
}
refresh();
setInterval(refresh, 750);
