export function originFrontierUsage(trace, ids, warm) {
  if (!Object.hasOwn(trace, "origin_requests")) return null;
  if (!Array.isArray(trace.origin_requests)) {
    throw new Error("origin_requests must be an array");
  }
  const epochs = localityEpochs(trace, ids, warm);
  const events = orderedOriginEvents(trace, epochs[0].startedAt);
  return epochs.reduce((total, epoch) => addUsage(total, countEpoch(events, epoch)), {
    far_bytes: 0,
    far_starts: 0,
  });
}

function localityEpochs(trace, ids, warm) {
  const recorded = (trace.focus_locality_epochs ?? []).map((epoch) => {
    return recordedEpoch(epoch, ids);
  });
  if (!warm) return requireRecordedEpochs(recorded);
  const initial = initialEpoch(ids, warm, recorded[0]);
  return [initial, ...recorded];
}

function requireRecordedEpochs(recorded) {
  if (recorded.length === 0) throw new Error("focus locality evidence is missing");
  return recorded;
}

function initialEpoch(ids, warm, firstRecorded) {
  return {
    startedAt: warm.focus_started_at_epoch_ms,
    endedAt: firstRecorded?.startedAt ?? null,
    startedAfter: null,
    endedThrough: firstRecorded?.startedAfter ?? null,
    protectedIds: ids.slice(0, warm.protected_count),
    farIds: ids.slice(warm.protected_count),
    baseline: warm.baseline_bytes,
    minimum: warm.minimum_bytes,
  };
}

function recordedEpoch(epoch, ids) {
  const index = ids.indexOf(epoch.focus_id);
  const expected = ids.slice(index, index + 4);
  if (index < 0 || !sameIds(epoch.protected_ids, expected)) {
    throw new Error("focus locality protected IDs are invalid");
  }
  return {
    startedAt: requireFinite(epoch.started_at_epoch_ms, "focus locality start"),
    endedAt: optionalFinite(epoch.ended_at_epoch_ms, "focus locality end"),
    startedAfter: requireBoundary(epoch.started_after_origin_ordinal),
    endedThrough: optionalBoundary(epoch.ended_through_origin_ordinal),
    protectedIds: expected,
    farIds: ids.slice(index + expected.length),
    baseline: epoch.baseline_bytes,
    minimum: requireFinite(epoch.minimum_bytes, "focus locality minimum"),
  };
}

function countEpoch(events, epoch) {
  const input = {
    ...epoch,
    protectedSet: new Set(epoch.protectedIds),
    farSet: new Set(epoch.farIds),
    totals: {...epoch.baseline},
  };
  let usage = {far_bytes: 0, far_starts: 0};
  for (const event of events.filter((entry) => belongsToEpoch(entry, epoch))) {
    if (frontierReady(input.totals, input.protectedIds, input.minimum)) break;
    usage = addUsage(usage, eventUsage(event, input.farSet));
    creditProtected(input, event);
  }
  return usage;
}

function belongsToEpoch(event, epoch) {
  if (event.at_ms < epoch.startedAt) return false;
  if (epoch.startedAfter !== null && event.ordinal <= epoch.startedAfter) return false;
  if (epoch.endedThrough !== null) return event.ordinal <= epoch.endedThrough;
  return epoch.endedAt === null || event.at_ms < epoch.endedAt;
}

function eventUsage(event, farSet) {
  if (!farSet.has(event.id)) return {far_bytes: 0, far_starts: 0};
  return {
    far_bytes: event.kind === "chunk" ? event.bytes : 0,
    far_starts: event.kind === "start" ? 1 : 0,
  };
}

export function orderedOriginEvents(trace, startedAt) {
  requireFinite(startedAt, "origin focus start");
  const events = trace.origin_requests.flatMap((request) => {
    const id = trace.video_ids?.[request.video];
    return [...startEvent(request, id, startedAt), ...chunkEvents(request, id, startedAt)];
  });
  requireUniqueOrdinals(events);
  return events.sort((left, right) => left.ordinal - right.ordinal);
}

function startEvent(request, id, startedAt) {
  const body = isBodyRequest(request);
  if (request.start_ordinal === null && !body) return [];
  const described = request.started_at_ms !== undefined || request.start_ordinal !== undefined;
  if (!described && !body) return [];
  const at = requireFinite(request.started_at_ms, "origin start timestamp");
  if (at < startedAt) return [];
  return [{kind: "start", at_ms: at,
    ordinal: requireOrdinal(request.start_ordinal, "origin start ordinal"),
    id: requireId(id)}];
}

function isBodyRequest(request) {
  return request.method === "GET" || request.bytes_sent > 0
    || (Array.isArray(request.chunk_events) && request.chunk_events.length > 0);
}

function chunkEvents(request, id, startedAt) {
  if (!Array.isArray(request.chunk_events ?? [])) {
    throw new Error("origin chunk events must be an array");
  }
  return (request.chunk_events ?? []).flatMap((event) => {
    const at = requireFinite(event.at_ms, "origin chunk timestamp");
    if (at < startedAt) return [];
    return [{kind: "chunk", at_ms: at,
      ordinal: requireOrdinal(event.ordinal, "origin chunk ordinal"),
      bytes: requireBytes(event.bytes), id: requireId(id)}];
  });
}

function creditProtected(input, event) {
  if (event.kind !== "chunk" || !input.protectedSet.has(event.id)) return;
  input.totals[event.id] = (input.totals[event.id] ?? 0) + event.bytes;
}

function frontierReady(totals, ids, minimum) {
  return ids.length > 0 && ids.every((id) => (totals[id] ?? 0) >= minimum);
}

function addUsage(left, right) {
  return {far_bytes: left.far_bytes + right.far_bytes,
    far_starts: left.far_starts + right.far_starts};
}

function requireUniqueOrdinals(events) {
  const unique = new Set(events.map((event) => event.ordinal));
  if (unique.size !== events.length) throw new Error("origin event ordinals must be unique");
}

function requireOrdinal(value, label) {
  if (!Number.isInteger(value) || value < 0) throw new Error(`${label} is invalid`);
  return value;
}

function requireBoundary(value) {
  if (!Number.isInteger(value) || value < -1) throw new Error("focus locality ordinal is invalid");
  return value;
}

function optionalBoundary(value) {
  return value === undefined ? null : requireBoundary(value);
}

function requireFinite(value, label) {
  if (!Number.isFinite(value)) throw new Error(`${label} is invalid`);
  return value;
}

function optionalFinite(value, label) {
  return value === undefined ? null : requireFinite(value, label);
}

function requireBytes(value) {
  if (!Number.isFinite(value) || value < 0) throw new Error("origin chunk bytes are invalid");
  return value;
}

function requireId(value) {
  if (!value) throw new Error("origin video ID is unknown");
  return value;
}

function sameIds(actual, expected) {
  return Array.isArray(actual) && actual.length === expected.length
    && actual.every((id, index) => id === expected[index]);
}
