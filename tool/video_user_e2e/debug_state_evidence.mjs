export function compactDebugState(state) {
  if (!state || typeof state !== "object") return state;
  const {adaptive_plans: _plans, ...telemetry} = state;
  return telemetry;
}
