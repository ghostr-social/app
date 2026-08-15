function renderDeliveryActivity(state) {
  renderActivity(state.videos);
  renderDeliveryLanes(state);
}

function renderDeliveryLanes(state) {
  const target = document.getElementById("delivery-lanes");
  const plan = state.adaptive_plans.at(-1);
  const videos = laneVideos(state.videos, plan);
  target.replaceChildren();
  videos.forEach((video) => target.append(deliveryLane(video, plan)));
  if (!videos.length) showEmptyLanes(target);
  const active = activeConnections(state.connections);
  document.getElementById("parallel-count").textContent =
    active > 0 ? `${active} simultaneous` : "Idle";
}

function laneVideos(videos, plan) {
  const ranked = plan?.ready_reserve?.candidates?.map((item) => item.post_id) || [];
  const planned = [...(plan?.allocations || []), ...(plan?.retained || [])]
    .map((item) => item.post_id);
  const ids = [...new Set([...ranked, ...planned])];
  const byId = new Map(videos.map((video) => [video.id, video]));
  const selected = ids.map((id) => byId.get(id)).filter(Boolean);
  return (selected.length ? selected : videos).slice(0, 5);
}

function deliveryLane(video, plan) {
  const row = document.createElement("div");
  const status = laneStatus(video, plan);
  const rate = rates.get(video.id) || 0;
  row.className = "delivery-lane";
  row.dataset.status = status;
  row.append(laneHeader(video, status), laneProgress(video), laneDetail(video, rate));
  return row;
}

function laneHeader(video, status) {
  const header = document.createElement("header");
  const name = document.createElement("span");
  const state = document.createElement("b");
  name.textContent = video.title || shortId(video.id);
  state.textContent = status;
  header.append(name, state);
  return header;
}

function laneProgress(video) {
  const progress = document.createElement("progress");
  progress.max = 1;
  progress.value = video.progress || 0;
  progress.setAttribute("aria-label", `${video.id} retrieval progress`);
  return progress;
}

function laneDetail(video, rate) {
  const detail = document.createElement("small");
  const present = document.createElement("span");
  const speed = document.createElement("span");
  present.textContent = `${bytes(video.downloaded_bytes)} present`;
  speed.textContent = rate > 1 ? `${bytes(rate)}/s` : `${Math.round((video.progress || 0) * 100)}%`;
  detail.append(present, speed);
  return detail;
}

function laneStatus(video, plan) {
  if ((rates.get(video.id) || 0) > 1) return "receiving";
  const reserve = plan?.ready_reserve?.candidates?.find((item) => item.post_id === video.id);
  if (reserve?.status === "ready") return "ready";
  const work = [...(plan?.allocations || []), ...(plan?.retained || [])];
  return work.some((item) => item.post_id === video.id) ? "scheduled" : reserve?.status || video.status;
}

function activeConnections(connections) {
  return connections.reduce((sum, connection) => sum + connection.active, 0);
}

function showEmptyLanes(target) {
  const empty = document.createElement("p");
  empty.className = "empty";
  empty.textContent = "Waiting for focused videos";
  target.append(empty);
}
