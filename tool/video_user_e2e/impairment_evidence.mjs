export function createEvidenceSender(input) {
  return (action) => applyImpairment({...input, action});
}

async function applyImpairment(input) {
  const before = await readState(input.read);
  await input.send(input.action);
  const appliedAt = (input.now ?? Date.now)();
  const after = await readState(input.read);
  input.evidence.push(receipt(input, appliedAt, before, after));
}

async function readState(read) {
  return read ? read() : undefined;
}

function receipt(input, appliedAt, before, after) {
  return {
    kind: input.action.kind,
    payload: structuredClone(input.action.payload),
    applied_at_epoch_ms: appliedAt,
    ...(input.startedAt === undefined ? {} : {at_ms: appliedAt - input.startedAt}),
    ...(input.read ? {before, after} : {}),
  };
}
