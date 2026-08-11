export function requireQoeTargets(metrics, targets) {
  requireWarm(metrics, targets);
  requirePlayback(metrics, targets);
  requireDelivery(metrics, targets);
  requireLocality(metrics, targets);
}

function requireWarm(metrics, targets) {
  if (!Object.hasOwn(metrics, "warm_prefetch_latency_ms")) return;
  requireAtMost(metrics.warm_prefetch_latency_ms, targets.warm_prefetch_latency_ms,
    "warm-prefetch readiness");
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
  requireAtLeast(metrics.ahead_prefetch_bytes, targets.ahead_prefetch_min_bytes,
    "ahead prefetch");
  requireAtMost(metrics.ahead_prefetch_bytes, targets.ahead_prefetch_max_bytes,
    "ahead prefetch");
  requireAtMost(metrics.duplicate_completed_origin_bytes,
    targets.duplicate_completed_origin_bytes, "duplicate completed origin bytes");
}

function requireLocality(metrics, targets) {
  requireAtMost(metrics.far_ahead_before_frontier_bytes,
    targets.far_ahead_before_frontier_bytes, "far-ahead before protected frontier");
  requireAtMost(metrics.far_ahead_request_starts_before_frontier,
    targets.far_ahead_request_starts_before_frontier,
    "far-ahead request starts before protected frontier");
}

function requireAtMost(actual, expected, label) {
  if (!Number.isFinite(actual) || actual > expected) {
    throw new Error(`${label} ${actual} exceeds ${expected}`);
  }
}

function requireAtLeast(actual, expected, label) {
  if (!Number.isFinite(actual) || actual < expected) {
    throw new Error(`${label} ${actual} is below ${expected}`);
  }
}
