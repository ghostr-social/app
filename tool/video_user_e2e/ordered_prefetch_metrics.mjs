import {originFrontierUsage} from "./origin_frontier_metrics.mjs";

export function farAheadBeforeFrontier(trace) {
  const warm = trace.warm_prefetch;
  const ids = trace.ordered_video_ids ?? warm?.ordered_ids ?? [];
  if (ids.length === 0) return 0;
  const origin = originFrontierUsage(trace, ids, warm);
  if (origin !== null) return origin.far_bytes;
  if (!warm?.baseline_bytes) return 0;
  const protectedIds = ids.slice(0, warm.protected_count);
  const farIds = ids.slice(warm.protected_count);
  if (frontierReady({downloaded_bytes: warm.baseline_bytes}, protectedIds, warm)) return 0;
  return sampledFarAhead(warm, protectedIds, farIds);
}

export function farAheadRequestStartsBeforeFrontier(trace) {
  const warm = trace.warm_prefetch;
  const ids = trace.ordered_video_ids ?? warm?.ordered_ids ?? [];
  if (ids.length === 0) return 0;
  return originFrontierUsage(trace, ids, warm)?.far_starts ?? 0;
}

function sampledFarAhead(warm, protectedIds, farIds) {
  let maximum = 0;
  for (const sample of warm.samples ?? []) {
    if (frontierReady(sample, protectedIds, warm)) break;
    maximum = Math.max(maximum, totalGain(sample, farIds, warm.baseline_bytes));
  }
  return maximum;
}

export function warmAheadBytes(trace) {
  const warm = trace.warm_prefetch;
  const ids = trace.ordered_video_ids ?? warm?.ordered_ids ?? [];
  if (!warm?.baseline_bytes || ids.length < 2) return 0;
  const aheadIds = ids.slice(1);
  return Math.max(0, ...(warm.samples ?? []).map((sample) => {
    return totalGain(sample, aheadIds, warm.baseline_bytes);
  }));
}

function frontierReady(sample, ids, warm) {
  return ids.length > 0 && ids.every((id) => {
    return (sample.downloaded_bytes[id] ?? 0) >= warm.minimum_bytes;
  });
}

function totalGain(sample, ids, baseline) {
  return ids.reduce((total, id) => total + byteGain(sample, id, baseline), 0);
}

function byteGain(sample, id, baseline) {
  return Math.max(0, (sample.downloaded_bytes[id] ?? 0) - (baseline[id] ?? 0));
}
