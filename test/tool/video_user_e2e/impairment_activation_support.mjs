export const STARTED_AT = 10_000;

export function activationTrace(scenario) {
  return {
    scenario,
    started_at_epoch_ms: STARTED_AT,
    video_ids: {v0: "id-0", v1: "id-1", v2: "id-2", v3: "id-3"},
    clicks: [
      {id: "id-0", at_ms: 0},
      {id: "id-1", at_ms: 800},
      {id: "id-2", at_ms: 1_600},
      {id: "id-3", at_ms: 4_200},
    ],
    impairments: [],
    origin_requests: [],
    samples: [],
  };
}

export function storageSample(at_ms, used_bytes) {
  return {at_ms, state: {storage: {used_bytes}}};
}

export function networkState(bandwidth_kbps, active = 1, latency_ms = 0,
  max_connections_per_host = 3) {
  return {
    network: {bandwidth_kbps, latency_ms, max_connections_per_host},
    connections: [{host: "127.0.0.1", active}],
    videos: [{id: "id-2", downloaded_bytes: 65_536, total_bytes: 370_912}],
  };
}
