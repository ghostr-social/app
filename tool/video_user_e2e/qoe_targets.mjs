const KIB = 1_024;
const MIB = KIB * KIB;

export const ORDERED_PREFETCH_TARGETS = Object.freeze({
  protected_count: 4,
  minimum_bytes: 48 * KIB,
  latency_ms: 4_000,
  far_origin_body_bytes: 0,
  far_origin_request_starts: 0,
});

export const QOE_TARGETS = Object.freeze({
  warm_prefetch_latency_ms: ORDERED_PREFETCH_TARGETS.latency_ms,
  startup_latency_ms: 2_000,
  focus_switch_latency_ms: 1_500,
  rebuffer_ratio: 0.01,
  cancellation_waste_bytes: 3 * 64 * KIB,
  ahead_prefetch_min_bytes: ORDERED_PREFETCH_TARGETS.minimum_bytes,
  ahead_prefetch_max_bytes: 3 * MIB,
  far_ahead_before_frontier_bytes: 0,
  far_ahead_request_starts_before_frontier: 0,
  duplicate_completed_origin_bytes: 0,
  protected_transition_latency_ms: 500,
});
