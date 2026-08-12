export function requireQoeTargets(metrics, targets) {
  requirePlayback(metrics, targets);
  requireDelivery(metrics, targets);
}

function requirePlayback(metrics, targets) {
  requireAtMost(metrics.startup_latency_ms, targets.startup_latency_ms, "startup latency");
  requireAtMost(metrics.focus_switch_latency_ms, targets.focus_switch_latency_ms,
    "focus-switch latency");
  requireAtMost(metrics.rebuffer_ratio, targets.rebuffer_ratio, "rebuffer ratio");
  requireAtMost(metrics.protected_transition_latency_ms,
    targets.protected_transition_latency_ms, "protected transition latency");
}

function requireDelivery(metrics, targets) {
  requireAtMost(metrics.cancellation_waste_bytes, targets.cancellation_waste_bytes,
    "cancellation waste");
  requireAtMost(metrics.ahead_prefetch_bytes, targets.ahead_prefetch_max_bytes,
    "ahead prefetch");
  requireAtMost(metrics.duplicate_completed_origin_bytes,
    targets.duplicate_completed_origin_bytes, "duplicate completed origin bytes");
}

function requireAtMost(actual, expected, label) {
  if (!Number.isFinite(actual) || actual > expected) {
    throw new Error(`${label} ${actual} exceeds ${expected}`);
  }
}
