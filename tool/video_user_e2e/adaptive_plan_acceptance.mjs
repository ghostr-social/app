import {duplicateCompletedOriginBytes} from "./duplicate_origin_metrics.mjs";
import {peakParallelOriginVideos} from "./parallel_origin_metrics.mjs";
import {
  measureReadyReserve, requireReadyReserve,
} from "./ready_reserve_acceptance.mjs";

const AUTHORITIES = new Set(["playback_critical", "transition", "speculative"]);
const ALLOCATION_REASONS = new Set([
  "current_stall_prevention",
  "current_buffer_reserve",
  "likely_next_transition",
  "rapid_navigation_coverage",
  "media_bootstrap",
  "media_layout_discovery",
  "next_startability",
  "useful_commitment",
]);
const DEMANDS = new Set(["expand", "hold"]);

export function requireAdaptivePlanEvidence(trace) {
  const plans = requirePlans(trace.adaptive_plans);
  requireOrderedRevisions(plans);
  for (const plan of plans) requirePlan(plan);
  requireOriginAdmissions(trace, plans);
  return measurePlans(plans);
}

export function measureAdaptivePlans(trace) {
  return measurePlans(requirePlans(trace.adaptive_plans));
}

export function requireAdaptiveBaseline(trace) {
  const metrics = requireAdaptivePlanEvidence(trace);
  const admitted = new Set(trace.adaptive_plans.flatMap((plan) => {
    return plan.allocations.map((work) => work.post_id);
  }));
  if (admitted.size < 2) throw new Error("healthy adaptive coverage did not expand");
  const duplicate = duplicateCompletedOriginBytes(trace.origin_requests);
  if (duplicate !== 0) throw new Error(`duplicate completed origin bytes ${duplicate}`);
  const parallel = peakParallelOriginVideos(trace.origin_requests);
  if (parallel < 2) throw new Error("adaptive baseline did not retrieve videos in parallel");
  return {
    ...metrics,
    duplicate_completed_origin_bytes: duplicate,
    peak_parallel_origin_videos: parallel,
  };
}

function measurePlans(plans) {
  const frontierSizes = plans.map(frontierSize);
  return {
    plan_revision_count: plans.length,
    frontier_sizes: frontierSizes,
    minimum_frontier_size: Math.min(...frontierSizes),
    maximum_frontier_size: Math.max(...frontierSizes),
    ready_reserve: measureReadyReserve(plans),
  };
}

function frontierSize(plan) {
  const posts = new Set([...plan.allocations, ...plan.retained].map((work) => work.post_id));
  return posts.size;
}

function requirePlans(plans) {
  if (!Array.isArray(plans) || plans.length === 0) {
    throw new Error("adaptive plan evidence is missing");
  }
  return plans;
}

function requireOrderedRevisions(plans) {
  let previous = {revision: 0, observed_at_ms: 0};
  for (const plan of plans) {
    requirePositiveInteger(plan.revision, "plan revision");
    requireNonnegativeInteger(plan.observed_at_ms, "plan observation time");
    if (plan.revision <= previous.revision || plan.observed_at_ms < previous.observed_at_ms) {
      throw new Error("adaptive plan revisions are out of order");
    }
    previous = plan;
  }
}

function requirePlan(plan) {
  requireEnum(plan.discovery_demand, DEMANDS, "discovery demand");
  requireReadyReserve(plan);
  requireArray(plan.allocations, "plan allocations");
  requireArray(plan.retained, "retained allocations");
  requireArray(plan.evictions, "plan evictions");
  for (const allocation of plan.allocations) requireAllocation(allocation);
  for (const retained of plan.retained) requireRetained(retained);
  for (const eviction of plan.evictions) requireEviction(eviction);
}

function requireAllocation(allocation) {
  requireWorkIdentity(allocation);
  requirePositive(allocation.expected_playable_gain_ms, "expected playable gain");
  requireUtility(allocation.utility, allocation.expected_playable_gain_ms);
  requireEnum(allocation.authority, AUTHORITIES, "preemption authority");
  requireNonnegativeInteger(allocation.commitment_until_ms, "commitment deadline");
  requireEnum(allocation.reason, ALLOCATION_REASONS, "allocation reason");
}

function requireUtility(utility, gain) {
  if (!utility || utility.additional_playable_ms !== gain) {
    throw new Error("allocation utility playable gain is invalid");
  }
  requireProbability(utility.view_probability);
  requirePositive(utility.expected_delivery_ms, "expected delivery time");
  requireNonnegative(utility.score, "utility score");
}

function requireRetained(retained) {
  requireWorkIdentity(retained);
  requireNonnegativeInteger(retained.committed_until_ms, "retained commitment deadline");
  requireEnum(retained.reason, ALLOCATION_REASONS, "retained allocation reason");
}

function requireEviction(eviction) {
  requireString(eviction.post_id, "eviction post");
  requireRange(eviction.range);
  requireNonnegative(eviction.expected_playable_loss_ms, "expected playable loss");
  requireEnum(eviction.reason, new Set(["storage_pressure"]), "eviction reason");
}

function requireWorkIdentity(work) {
  requireString(work.post_id, "allocation post");
  requireRange(work.range);
  requireString(work.source, "allocation source");
}

function requireRange(range) {
  if (!range || !Number.isSafeInteger(range.start) || !Number.isSafeInteger(range.end)
    || range.start < 0 || range.end <= range.start) {
    throw new Error("allocation byte range is invalid");
  }
}

function requireOriginAdmissions(trace, plans) {
  for (const request of trace.origin_requests ?? []) {
    if (request.method === "HEAD" || !(request.bytes_sent > 0)) continue;
    const post = trace.video_ids?.[request.video];
    if (!post || !admittedBefore(plans, request, post)) {
      throw new Error(`unadmitted origin range for ${post ?? request.video ?? "unknown"}`);
    }
  }
}

function admittedBefore(plans, request, post) {
  return plans.some((plan) => plan.observed_at_ms <= request.started_at_ms
    && plan.allocations.some((work) => exactAdmission(work, request, post)));
}

function exactAdmission(work, request, post) {
  return work.post_id === post && work.range.start === request.start
    && work.range.end === request.end;
}

function requireArray(value, label) {
  if (!Array.isArray(value)) throw new Error(`${label} is missing`);
}

function requireEnum(value, allowed, label) {
  if (!allowed.has(value)) throw new Error(`${label} is invalid`);
}

function requireString(value, label) {
  if (typeof value !== "string" || value.length === 0) throw new Error(`${label} is invalid`);
}

function requireProbability(value) {
  if (!Number.isFinite(value) || value < 0 || value > 1) {
    throw new Error("view probability is invalid");
  }
}

function requirePositive(value, label) {
  if (!Number.isFinite(value) || value <= 0) throw new Error(`${label} is invalid`);
}

function requireNonnegative(value, label) {
  if (!Number.isFinite(value) || value < 0) throw new Error(`${label} is invalid`);
}

function requirePositiveInteger(value, label) {
  if (!Number.isSafeInteger(value) || value <= 0) throw new Error(`${label} is invalid`);
}

function requireNonnegativeInteger(value, label) {
  if (!Number.isSafeInteger(value) || value < 0) throw new Error(`${label} is invalid`);
}
