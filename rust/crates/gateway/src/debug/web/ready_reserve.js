function renderReadyReserve(plans) {
  const plan = plans.at(-1);
  const reserve = plan?.ready_reserve;
  const candidates = document.getElementById("reserve-candidates");
  candidates.replaceChildren();
  if (!reserve) {
    showEmptyReserve(candidates);
    return;
  }
  document.getElementById("reserve-mode").textContent = plan.mode;
  document.getElementById("reserve-count").textContent =
    `${reserve.ready} ready · ${reserve.protected}/${reserve.target} protected`;
  document.getElementById("reserve-coverage").textContent =
    `${(reserve.ready_coverage_ms / 1000).toFixed(1)}s`;
  document.getElementById("reserve-risk").textContent =
    `${(reserve.underflow_risk_bps / 100).toFixed(2)}%`;
  document.getElementById("reserve-horizon").textContent =
    `${(reserve.recovery_horizon_ms / 1000).toFixed(2)}s`;
  reserve.candidates.forEach((candidate) => candidates.append(reserveRow(candidate)));
  if (!reserve.candidates.length) showEmptyReserve(candidates);
}

function reserveRow(candidate) {
  const row = document.createElement("div");
  const post = document.createElement("span");
  const status = document.createElement("b");
  row.className = "reserve-candidate";
  row.dataset.status = candidate.status;
  post.textContent = reserveShortId(candidate.post_id);
  status.textContent = candidate.status.replaceAll("_", " ");
  row.append(post, status);
  return row;
}

function reserveShortId(id) {
  return id.length > 24 ? `${id.slice(0, 12)}…${id.slice(-7)}` : id;
}

function showEmptyReserve(target) {
  const empty = document.createElement("p");
  empty.className = "empty";
  empty.textContent = "Waiting for a delivery plan";
  target.append(empty);
  document.getElementById("reserve-mode").textContent = "Waiting";
  document.getElementById("reserve-count").textContent = "—";
  document.getElementById("reserve-coverage").textContent = "—";
  document.getElementById("reserve-risk").textContent = "—";
  document.getElementById("reserve-horizon").textContent = "—";
}
