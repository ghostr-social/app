const STORAGE_BUDGET_BYTES = 2 * 1_024 * 1_024;
const STORAGE_MARGIN_BYTES = 64 * 1_024;
const PACKET_OBSERVATION_MS = 2_500;

const REQUIREMENTS = Object.freeze({
  bandwidth_drop: requireBandwidthDrop,
  high_rtt: (trace) => requireNetworkProfile(trace, {
    bandwidth_kbps: 2_500, latency_ms: 450, max_connections_per_host: 3,
  }, "high RTT"),
  packet_loss: requirePacketLoss,
  protected_transitions: (trace) => requireNetworkProfile(trace, {
    bandwidth_kbps: 2_500, latency_ms: 100, max_connections_per_host: 1,
  }, "protected-transition"),
  source_failure: requireSourceFailure,
  storage_pressure: requireStoragePressure,
});

export function requireImpairmentActivation(trace) {
  REQUIREMENTS[trace.scenario]?.(trace);
}

function requirePacketLoss(trace) {
  const clickIndex = trace.clicks.findIndex((click) => click.id === trace.video_ids?.v2);
  const click = trace.clicks[clickIndex];
  const next = trace.clicks[clickIndex + 1];
  if (!observedPacketWindow(click, next)) {
    throw new Error("packet loss did not observe clicked v2 for 2.5 seconds");
  }
  const failures = packetFailuresBefore(trace, next.at_ms);
  if (!twiceImpairedWhileClicked(trace, click, failures)) {
    throw new Error("packet loss did not inject two failures into clicked v2");
  }
}

function observedPacketWindow(click, next) {
  return Boolean(click && next) && next.at_ms - click.at_ms >= PACKET_OBSERVATION_MS;
}

function twiceImpairedWhileClicked(trace, click, failures) {
  const clickedAt = trace.started_at_epoch_ms + click.at_ms;
  return failures.length >= 2
    && failures.some((request) => request.closed_at_ms >= clickedAt);
}

function packetFailuresBefore(trace, endAt) {
  const deadline = trace.started_at_epoch_ms + endAt;
  return trace.origin_requests.filter((request) => {
    return request.video === "v2" && request.injected_failure
      && request.closed_at_ms <= deadline;
  });
}

function requireSourceFailure(trace) {
  const selected = trace.clicks.find((click) => !click.superseded)?.id;
  const video = Object.entries(trace.video_ids ?? {})
    .find(([, id]) => id === selected)?.[0];
  const primary = trace.origin_requests.find(
    (request) => request.id === `${video}-primary` && request.failed_status === 503,
  );
  const mirror = trace.origin_requests.find((request) => {
    return request.id === `${video}-mirror` && request.completed && request.bytes_sent > 0
      && request.started_at_ms >= (primary?.closed_at_ms ?? Number.POSITIVE_INFINITY);
  });
  if (!primary || !mirror) {
    throw new Error("selected video did not complete a mirror body after primary 503");
  }
}

function requireStoragePressure(trace) {
  const released = requireStorageControls(trace.impairments);
  const before = storedBytes(trace, (sample) => sample.at_ms < released.at_ms);
  const highWater = requireStoragePark(before);
  const resumed = storedBytes(trace, (sample) => sample.at_ms >= released.at_ms)
    .some((bytes) => bytes > highWater);
  if (!resumed) throw new Error("storage delivery did not resume after budget release");
}

function requireStorageControls(evidence) {
  const controls = evidence.filter((entry) => entry.kind === "storage");
  const limited = controls.find((entry) => entry.payload.budget_bytes === STORAGE_BUDGET_BYTES);
  const released = controls.find((entry) => releasedStorageControl(entry));
  if (!limited || !released) throw new Error("storage pressure controls were not applied");
  return released;
}

function releasedStorageControl(entry) {
  return entry.payload.budget_bytes > STORAGE_BUDGET_BYTES && Number.isFinite(entry.at_ms);
}

function requireStoragePark(bytes) {
  const highWater = Math.max(...bytes);
  const parked = bytes.filter((value) => value === highWater).length >= 2;
  if (!nearStorageBudget(highWater) || !parked) {
    throw new Error("storage delivery did not park at the 2 MiB budget");
  }
  return highWater;
}

function nearStorageBudget(bytes) {
  return bytes <= STORAGE_BUDGET_BYTES
    && bytes >= STORAGE_BUDGET_BYTES - STORAGE_MARGIN_BYTES;
}

function storedBytes(trace, accept) {
  return trace.samples.filter(accept)
    .map((sample) => sample.state?.storage?.used_bytes)
    .filter(Number.isFinite);
}

function requireBandwidthDrop(trace) {
  const drop = requireBandwidthControls(trace.impairments);
  if (!activeIncompleteDelivery(drop.after)
    || drop.after?.network?.bandwidth_kbps !== 700) {
    throw new Error("bandwidth drop was not applied during active incomplete delivery");
  }
}

function requireNetworkProfile(trace, profile, label) {
  const active = activationSamples(trace).some((sample) => {
    return matchesNetwork(sample.state?.network, profile)
      && activeIncompleteDelivery(sample.state ?? {});
  });
  if (!active) {
    throw new Error(`${label} profile was not sampled during active incomplete delivery`);
  }
}

function activationSamples(trace) {
  return [...(trace.warm_prefetch?.samples ?? []), ...(trace.samples ?? [])];
}

function matchesNetwork(actual, expected) {
  return actual?.bandwidth_kbps === expected.bandwidth_kbps
    && actual.latency_ms === expected.latency_ms
    && actual.max_connections_per_host === expected.max_connections_per_host;
}

function requireBandwidthControls(evidence) {
  const controls = evidence.filter((entry) => entry.kind === "network");
  const drop = controls.find((entry) => entry.payload.bandwidth_kbps === 700);
  const recovery = controls.find((entry) => recoveredAfter(entry, drop));
  if (!drop || !recovery || recovery.after?.network?.bandwidth_kbps !== 2_500) {
    throw new Error("bandwidth recovery was not applied after the drop");
  }
  return drop;
}

function recoveredAfter(entry, drop) {
  return entry.payload.bandwidth_kbps === 2_500 && entry.at_ms > (drop?.at_ms ?? Infinity);
}

function activeIncompleteDelivery(state) {
  const active = state.connections?.some((connection) => connection.active > 0);
  const incomplete = state.videos?.some((video) => {
    return video.downloaded_bytes < video.total_bytes;
  });
  return active && incomplete;
}
