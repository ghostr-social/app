export const IMPAIRMENT_SCENARIOS = deepFreeze({
  ordered_prefetch: {},
  bandwidth_drop: {
    network: {steps: [
      {at_ms: 0, bandwidth_kbps: 2_500},
      {at_ms: 1_500, bandwidth_kbps: 700},
      {at_ms: 4_500, bandwidth_kbps: 2_500},
    ]},
  },
  packet_loss: {
    origin: {
      abort_first_attempts: {video: "v2", count: 2},
      abort_after_bytes: 128 * 1_024,
    },
  },
  high_rtt: {
    network: {latency_ms: 450, bandwidth_kbps: 2_500},
  },
  rapid_swipes: {
    focus: [
      {at_ms: 0, index: 0},
      {at_ms: 200, index: 1},
      {at_ms: 400, index: 2},
      {at_ms: 600, index: 3},
    ],
  },
  storage_pressure: {
    storage: {budget_bytes: 2 * 1_024 * 1_024, release_at_ms: 3_000},
  },
  source_failure: {
    origin: {fail_source: "primary", status: 503},
  },
  protected_transitions: {
    network: {
      bandwidth_kbps: 2_500,
      latency_ms: 100,
      max_connections_per_host: 1,
    },
  },
});

function deepFreeze(value) {
  for (const child of Object.values(value)) {
    if (child && typeof child === "object") deepFreeze(child);
  }
  return Object.freeze(value);
}
