export function duplicateCompletedOriginBytes(requests = []) {
  const rangesBySource = new Map();
  let duplicateBytes = 0;
  for (const request of eligibleRequests(requests)) {
    const previous = rangesBySource.get(request.id) ?? [];
    duplicateBytes += overlapWithRanges(request, previous);
    rangesBySource.set(request.id, includeRange(previous, request));
  }
  return duplicateBytes;
}

function eligibleRequests(requests) {
  return requests.filter(isEligible)
    .sort((left, right) => left.start_ordinal - right.start_ordinal);
}

function isEligible(request) {
  return isSuccessful(request) && hasBodyIdentity(request) && hasValidRange(request);
}

function isSuccessful(request) {
  return request.completed === true
    && !request.injected_failure
    && request.failed_status == null;
}

function hasBodyIdentity(request) {
  return Number.isInteger(request.start_ordinal) && typeof request.id === "string";
}

function hasValidRange(request) {
  return Number.isSafeInteger(request.start) && Number.isSafeInteger(request.end)
    && request.start >= 0 && request.end > request.start;
}

function overlapWithRanges(request, ranges) {
  return ranges.reduce((total, range) => total + overlap(request, range), 0);
}

function overlap(left, right) {
  const start = Math.max(left.start, right.start);
  const end = Math.min(left.end, right.end);
  return Math.max(0, end - start);
}

function includeRange(ranges, request) {
  const ordered = [...ranges, {start: request.start, end: request.end}]
    .sort((left, right) => left.start - right.start);
  const merged = [];
  for (const range of ordered) mergeInto(merged, range);
  return merged;
}

function mergeInto(merged, range) {
  const previous = merged.at(-1);
  if (!previous || previous.end < range.start) return merged.push({...range});
  previous.end = Math.max(previous.end, range.end);
}
