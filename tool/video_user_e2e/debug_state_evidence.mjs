export function compactDebugState(state) {
  if (!state || typeof state !== "object") return state;
  const {adaptive_plans: _plans, decisions: _decisions, evaluation: _evaluation,
    ...telemetry} = state;
  return telemetry;
}
