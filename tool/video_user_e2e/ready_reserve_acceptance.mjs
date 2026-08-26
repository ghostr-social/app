const MODES = new Set(["emergency", "safety", "normal"]);
const STATES = new Set([
  "unprepared", "ready", "structural", "in_flight", "probing", "preparing", "planned",
  "infeasible",
]);
const PROTECTED_STATES = new Set(["ready", "structural", "in_flight", "planned"]);

export function requireReadyReserve(plan) {
  if (!MODES.has(plan.mode)) throw new Error("control mode is invalid");
  const reserve = plan.ready_reserve;
  if (!reserve || !Array.isArray(reserve.candidates)) {
    throw new Error("ready reserve is missing");
  }
  for (const field of reserveFields()) requireInteger(reserve[field], field);
  if (reserve.underflow_risk_bps > 10_000) throw new Error("underflow risk is invalid");
  reserve.candidates.forEach(requireCandidate);
  requireCoherentCounts(reserve);
}

export function measureReadyReserve(plans) {
  return {
    maximum_target: maximum(plans, "target"),
    maximum_ready: maximum(plans, "ready"),
    maximum_structural: maximum(plans, "structural"),
    maximum_protected: maximum(plans, "protected"),
    maximum_coverage_ms: maximum(plans, "ready_coverage_ms"),
  };
}

function reserveFields() {
  return ["target", "ready", "structural", "protected", "recovery_horizon_ms",
    "underflow_risk_bps", "ready_coverage_ms"];
}

function requireCoherentCounts(reserve) {
  const ready = countState(reserve.candidates, (status) => status === "ready");
  const structural = countState(reserve.candidates, (status) => status === "structural");
  const protectedCount = countState(reserve.candidates, (status) => PROTECTED_STATES.has(status));
  if (ready !== reserve.ready || structural !== reserve.structural ||
      protectedCount !== reserve.protected) {
    throw new Error("ready reserve counters do not match candidates");
  }
}

function countState(candidates, predicate) {
  return candidates.filter((candidate) => predicate(candidate.status)).length;
}

function requireCandidate(candidate) {
  if (!candidate || typeof candidate.post_id !== "string" || !candidate.post_id) {
    throw new Error("reserve candidate post is invalid");
  }
  if (!STATES.has(candidate.status)) throw new Error("reserve candidate state is invalid");
}

function requireInteger(value, field) {
  if (!Number.isSafeInteger(value) || value < 0) throw new Error(`${field} is invalid`);
}

function maximum(plans, field) {
  return Math.max(...plans.map((plan) => plan.ready_reserve[field]));
}
