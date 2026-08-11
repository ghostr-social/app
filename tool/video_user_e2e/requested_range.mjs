export function requestedRange(header, total) {
  if (!header) return {start: 0, end: total, partial: false};
  const match = /^bytes=(\d+)-(\d*)$/.exec(header);
  if (!match) return null;
  return boundedRange(Number(match[1]), inclusiveEnd(match[2], total), total);
}

function inclusiveEnd(raw, total) {
  return raw ? Number(raw) : total - 1;
}

function boundedRange(start, inclusive, total) {
  if (!Number.isSafeInteger(start) || start >= total) return null;
  if (!Number.isSafeInteger(inclusive) || inclusive < start) return null;
  return {start, end: Math.min(inclusive + 1, total), partial: true};
}
