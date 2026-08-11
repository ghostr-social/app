export function createFocusLocalityClick(input) {
  const epochs = input.trace.focus_locality_epochs ??= [];
  return async (id) => {
    const state = await input.read();
    const boundary = latestOriginOrdinal(input.originRequests);
    const startedAt = (input.now ?? Date.now)();
    closePreviousEpoch(epochs.at(-1), startedAt, boundary);
    epochs.push(createEpoch(input, id, state, startedAt, boundary));
    await input.click(id);
  };
}

export function recordInitialFocusLocality(input) {
  const epochs = input.trace.focus_locality_epochs ??= [];
  const boundary = latestOriginOrdinal(input.originRequests);
  const epoch = createEpoch(input, input.id, input.state, input.startedAt, boundary);
  epochs.push({...epoch, pre_click: true});
}

function createEpoch(input, id, state, startedAt, boundary) {
  const index = input.orderedIds.indexOf(id);
  if (index < 0) throw new Error(`focus locality received unknown video ${id}`);
  return {
    focus_id: id,
    started_at_epoch_ms: startedAt,
    started_after_origin_ordinal: boundary,
    protected_ids: input.orderedIds.slice(index, index + input.protectedCount),
    baseline_bytes: downloadedBytes(state, input.orderedIds),
    minimum_bytes: input.minimumBytes,
  };
}

function closePreviousEpoch(epoch, endedAt, boundary) {
  if (!epoch) return;
  epoch.ended_at_epoch_ms = endedAt;
  epoch.ended_through_origin_ordinal = boundary;
}

function downloadedBytes(state, ids) {
  const videos = new Map((state?.videos ?? []).map((video) => [video.id, video.downloaded_bytes]));
  return Object.fromEntries(ids.map((id) => [id, videos.get(id) ?? 0]));
}

function latestOriginOrdinal(requests) {
  return requests.reduce((latest, request) => {
    const values = [request.start_ordinal, ...(request.chunk_events ?? []).map(eventOrdinal)];
    return Math.max(latest, ...values.filter(Number.isInteger));
  }, -1);
}

function eventOrdinal(event) {
  return event.ordinal;
}
