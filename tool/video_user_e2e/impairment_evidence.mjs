import {compactDebugState} from "./debug_state_evidence.mjs";

export function createEvidenceSender(input) {
  return (action) => applyImpairment({...input, action});
}

async function applyImpairment(input) {
  const before = await readState(input.read);
  const now = input.now ?? Date.now;
  const requestedAt = now();
  await input.send(input.action);
  const appliedAt = now();
  const after = await readState(input.read);
  input.evidence.push(receipt(input, requestedAt, appliedAt, before, after));
}

async function readState(read) {
  return read ? compactDebugState(await read()) : undefined;
}

function receipt(input, requestedAt, appliedAt, before, after) {
  return {
    kind: input.action.kind,
    payload: structuredClone(input.action.payload),
    requested_at_epoch_ms: requestedAt,
    applied_at_epoch_ms: appliedAt,
    ...(input.startedAt === undefined ? {} : {at_ms: appliedAt - input.startedAt}),
    ...(input.read ? {before, after} : {}),
  };
}
