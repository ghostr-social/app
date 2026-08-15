export function peakParallelOriginVideos(requests = []) {
  const events = requests.flatMap(requestEvents).sort(compareEvents);
  const active = new Map();
  let peak = 0;
  for (const event of events) {
    updateActive(active, event);
    peak = Math.max(peak, active.size);
  }
  return peak;
}

function requestEvents(request, index) {
  if (request.method === "HEAD" || !request.video) return [];
  if (!Number.isFinite(request.started_at_ms) || !Number.isFinite(request.closed_at_ms)) return [];
  if (request.closed_at_ms < request.started_at_ms) return [];
  return [
    {at: request.started_at_ms, order: 1, video: request.video, index},
    {at: request.closed_at_ms, order: -1, video: request.video, index},
  ];
}

function compareEvents(left, right) {
  return left.at - right.at || left.order - right.order || left.index - right.index;
}

function updateActive(active, event) {
  const count = active.get(event.video) || 0;
  const next = count + event.order;
  if (next > 0) active.set(event.video, next);
  else active.delete(event.video);
}
