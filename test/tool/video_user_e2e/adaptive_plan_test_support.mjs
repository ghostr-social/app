export function adaptiveTrace(overrides = {}) {
  return {
    video_ids: {v0: "post-0", v1: "post-1"},
    adaptive_plans: [plan()],
    origin_requests: [],
    ...overrides,
  };
}

export function plan(overrides = {}) {
  return {
    revision: 1,
    observed_at_ms: 100,
    discovery_demand: "hold",
    allocations: [allocation()],
    retained: [],
    evictions: [],
    ...overrides,
  };
}

export function allocation(overrides = {}) {
  return {
    post_id: "post-0",
    range: {start: 0, end: 64 * 1_024},
    source: "http://127.0.0.1:4100/v0.mp4",
    expected_playable_gain_ms: 1_000,
    utility: {
      view_probability: 1,
      additional_playable_ms: 1_000,
      expected_delivery_ms: 100,
      score: 10,
    },
    authority: "playback_critical",
    commitment_until_ms: 5_000,
    reason: "current_stall_prevention",
    ...overrides,
  };
}

export function bodyRequest(overrides = {}) {
  return {
    video: "v0",
    method: "GET",
    start: 0,
    end: 64 * 1_024,
    started_at_ms: 150,
    bytes_sent: 64 * 1_024,
    ...overrides,
  };
}
