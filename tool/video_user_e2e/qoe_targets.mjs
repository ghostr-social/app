const KIB = 1_024;
const MIB = KIB * KIB;

export const QOE_TARGETS = Object.freeze({
  startup_latency_ms: 2_000,
  focus_switch_latency_ms: 1_500,
  rebuffer_ratio: 0.01,
  cancellation_waste_bytes: 3 * 64 * KIB,
  ahead_prefetch_max_bytes: 3 * MIB,
  duplicate_completed_origin_bytes: 0,
  protected_transition_latency_ms: 500,
});
