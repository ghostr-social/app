import {orderedOriginEvents} from "./origin_frontier_metrics.mjs";

export function measureOrderedPrefetch(trace) {
  const warm = requireWarm(trace.warm_prefetch);
  const ids = requireIds(trace.ordered_video_ids);
  const protectedIds = ids.slice(0, warm.protected_count);
  const ready = protectedIds.map((id) => warm.ready_bytes[id] ?? 0);
  const far = farOriginUsage(trace, ids.slice(warm.protected_count));
  return {
    protected_count: warm.protected_count,
    protected_readiness_entries: protectedIds.filter((id) => {
      return Object.hasOwn(warm.ready_bytes, id);
    }).length,
    warm_prefetch_latency_ms: warm.latency_ms,
    protected_prefetch_min_bytes: Math.min(...ready),
    ...far,
  };
}

export function requireOrderedPrefetchTargets(metrics, targets) {
  requireEqual(metrics.protected_count, targets.protected_count, "protected count");
  requireEqual(metrics.protected_readiness_entries, targets.protected_count,
    "protected readiness entries");
  requireAtMost(metrics.warm_prefetch_latency_ms, targets.latency_ms, "warm prefetch latency");
  requireAtLeast(metrics.protected_prefetch_min_bytes, targets.minimum_bytes, "protected bytes");
  requireAtMost(metrics.far_origin_body_bytes, targets.far_origin_body_bytes,
    "far origin body bytes");
  requireAtMost(metrics.far_origin_request_starts, targets.far_origin_request_starts,
    "far origin request starts");
}

function requireEqual(actual, expected, label) {
  if (actual !== expected) throw new Error(`${label} ${actual} does not equal ${expected}`);
}

function farOriginUsage(trace, farIds) {
  if (!Array.isArray(trace.origin_requests)) throw new Error("origin_requests must be an array");
  const far = new Set(farIds);
  return orderedOriginEvents(trace, 0).reduce((total, event) => {
    if (!far.has(event.id)) return total;
    if (event.kind === "start") total.far_origin_request_starts += 1;
    if (event.kind === "chunk") total.far_origin_body_bytes += event.bytes;
    return total;
  }, {far_origin_body_bytes: 0, far_origin_request_starts: 0});
}

function requireWarm(warm) {
  if (!warm?.ready_bytes || !Number.isFinite(warm.latency_ms)) {
    throw new Error("ordered prefetch readiness evidence is missing");
  }
  return warm;
}

function requireIds(ids) {
  if (!Array.isArray(ids) || ids.length === 0) throw new Error("ordered video IDs are missing");
  return ids;
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
