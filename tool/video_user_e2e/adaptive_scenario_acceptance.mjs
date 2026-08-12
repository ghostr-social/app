import {requireAdaptivePlanEvidence} from "./adaptive_plan_acceptance.mjs";

const OUTCOMES = Object.freeze({
  bandwidth_drop: (trace) => requireNetworkReplan(trace, (payload) => {
    return payload.bandwidth_kbps === 700;
  }),
  packet_loss: (trace) => requireNetworkReplan(trace, (payload) => {
    return payload.packet_loss_bps === 6_000;
  }),
  rapid_swipes: requireRapidCoverage,
  source_failure: requireSourceReallocation,
  storage_pressure: requireStorageContraction,
});

export function requireAdaptiveScenarioOutcome(trace) {
  requireAdaptivePlanEvidence(trace);
  OUTCOMES[trace.scenario]?.(trace);
}

function requireNetworkReplan(trace, impaired) {
  const receipts = (trace.impairments ?? []).filter((entry) => entry.kind === "network");
  const startIndex = receipts.findIndex((entry) => impaired(entry.payload));
  const start = requestedAt(receipts[startIndex]);
  const recovery = requestedAt(receipts[startIndex + 1]);
  const before = trace.adaptive_plans.filter((plan) => plan.observed_at_ms < start);
  const impairedPlans = trace.adaptive_plans.filter((plan) => {
    return plan.observed_at_ms >= start && plan.observed_at_ms < recovery;
  });
  if (!before.length || !impairedPlans.length) {
    throw new Error("network replan evidence is missing");
  }
  const stableEnd = Math.min(recovery, nextFocusEpoch(trace, start));
  const stablePlans = impairedPlans.filter((plan) => plan.observed_at_ms < stableEnd);
  const applied = receipts[startIndex]?.applied_at_epoch_ms ?? start;
  const appliedRecovery = receipts[startIndex + 1]?.applied_at_epoch_ms
    ?? Number.POSITIVE_INFINITY;
  requireNoRestartedOriginBytes(trace, applied, appliedRecovery);
  if (!networkDecisionChanged(before.at(-1), stablePlans, impairedPlans)) {
    throw new Error("network impairment did not change adaptive allocation evidence");
  }
}

function nextFocusEpoch(trace, start) {
  if (!Number.isFinite(trace.started_at_epoch_ms) || !Number.isFinite(start)) {
    return Number.POSITIVE_INFINITY;
  }
  const next = (trace.clicks ?? []).find((click) => {
    return trace.started_at_epoch_ms + click.at_ms > start;
  });
  return next ? trace.started_at_epoch_ms + next.at_ms : Number.POSITIVE_INFINITY;
}

function networkDecisionChanged(before, stablePlans, impairedPlans) {
  if (stablePlans.length && maximumBreadth(stablePlans) < frontierSize(before)) return true;
  const prior = [...before.allocations, ...before.retained];
  return impairedPlans.some((plan) => planWork(plan).some((work) => {
    const old = prior.find((candidate) => sameWork(work, candidate));
    return old && Number.isFinite(work.utility?.expected_delivery_ms)
      && work.utility.expected_delivery_ms !== old.utility?.expected_delivery_ms;
  }));
}

function planWork(plan) {
  return [...plan.allocations, ...plan.retained];
}

function requireNoRestartedOriginBytes(trace, start, end) {
  const paid = new Map();
  const requests = (trace.origin_requests ?? []).filter((request) => {
    return request.method !== "HEAD" && request.started_at_ms >= start
      && request.started_at_ms < end && request.bytes_sent > 0
      && request.completed === true && !request.injected_failure
      && request.failed_status == null;
  });
  for (const request of requests) recordPaidRange(paid, request);
}

function recordPaidRange(paid, request) {
  const range = {start: request.start,
    end: Math.min(request.end, request.start + request.bytes_sent)};
  const prior = paid.get(request.video) ?? [];
  if (prior.some((candidate) => overlaps(candidate, range))) {
    throw new Error("network replan restarted useful origin bytes");
  }
  paid.set(request.video, [...prior, range]);
}

function overlaps(left, right) {
  return left.start < right.end && right.start < left.end;
}

function requireRapidCoverage(trace) {
  const rapid = trace.adaptive_plans.some((plan) => plan.allocations.some((work) => {
    return work.reason === "rapid_navigation_coverage";
  }));
  if (!rapid) throw new Error("rapid navigation did not expand adaptive coverage");
}

function requireStorageContraction(trace) {
  const controls = (trace.impairments ?? []).filter((entry) => entry.kind === "storage");
  const limited = controls[0];
  const release = controls.find((entry) => {
    return entry.payload.budget_bytes > (limited?.payload.budget_bytes ?? Infinity);
  });
  const releaseAt = requestedAt(release);
  const before = trace.adaptive_plans.filter((plan) => {
    return plan.observed_at_ms < releaseAt;
  });
  const after = trace.adaptive_plans.filter((plan) => {
    return plan.observed_at_ms >= releaseAt;
  });
  if (!limited || !release || !before.length || !after.length) {
    throw new Error("storage allocation evidence is missing");
  }
  const constrained = frontierSize(before.at(-1));
  if (!after.some((plan) => frontierSize(plan) > constrained)) {
    throw new Error("storage pressure did not contract adaptive allocation");
  }
}

function requestedAt(receipt) {
  return receipt?.requested_at_epoch_ms
    ?? receipt?.applied_at_epoch_ms
    ?? Number.POSITIVE_INFINITY;
}

function requireSourceReallocation(trace) {
  const sources = new Map();
  for (const plan of trace.adaptive_plans) {
    for (const work of plan.allocations) {
      const key = `${work.post_id}:${work.range.start}:${work.range.end}`;
      const choices = sources.get(key) ?? new Set();
      choices.add(work.source);
      sources.set(key, choices);
    }
  }
  if (![...sources.values()].some((choices) => choices.size > 1)) {
    throw new Error("failed source did not reallocate an exact range");
  }
}

function maximumBreadth(plans) {
  return Math.max(...plans.map(frontierSize));
}

function frontierSize(plan) {
  return new Set([...plan.allocations, ...plan.retained].map((work) => work.post_id)).size;
}

function sameWork(left, right) {
  return left.post_id === right.post_id && left.source === right.source
    && left.range.start === right.range.start && left.range.end === right.range.end;
}
