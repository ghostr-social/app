import {poll} from "./wait.mjs";

export async function waitForWarmPrefetch(input) {
  const now = input.now ?? Date.now;
  const elapsed = now() - input.startedAt;
  requireInsideDeadline(elapsed, input.deadlineMs);
  const baseline = downloadedBytes(input.baseline, input.orderedIds);
  const evidence = initialEvidence(input, baseline);
  input.onEvidence?.(evidence);
  const sample = await (input.wait ?? poll)({
    read: () => readSample(input, evidence.samples, now),
    accept: (value) => protectedReady(value, input),
    timeoutMs: input.deadlineMs - elapsed,
    intervalMs: 100,
    label: "ordered protected-prefix warm prefetch",
    signal: input.signal,
  });
  completeEvidence(evidence, input, sample, now());
  requireInsideDeadline(evidence.latency_ms, evidence.deadline_ms);
  return evidence;
}

function initialEvidence(input, baseline) {
  return {
    focus_started_at_epoch_ms: input.startedAt,
    ordered_ids: input.orderedIds,
    protected_count: input.protectedCount,
    baseline_bytes: baseline,
    samples: [],
    minimum_bytes: input.minimumBytes,
    deadline_ms: input.deadlineMs,
  };
}

function completeEvidence(evidence, input, sample, finishedAt) {
  evidence.ready_bytes = protectedBytes(sample, input);
  evidence.latency_ms = finishedAt - input.startedAt;
}

async function readSample(input, samples, now) {
  const state = await input.read();
  const sample = {
    at_ms: now() - input.startedAt,
    downloaded_bytes: downloadedBytes(state, input.orderedIds),
  };
  samples.push(sample);
  return sample;
}

function protectedReady(sample, input) {
  return input.orderedIds.slice(0, input.protectedCount).every((id) => {
    return sample.downloaded_bytes[id] >= input.minimumBytes;
  });
}

function protectedBytes(sample, input) {
  return Object.fromEntries(input.orderedIds.slice(0, input.protectedCount)
    .map((id) => [id, sample.downloaded_bytes[id]]));
}

function downloadedBytes(state, ids) {
  const videos = new Map((state?.videos ?? []).map((video) => [video.id, video.downloaded_bytes]));
  return Object.fromEntries(ids.map((id) => [id, videos.get(id) ?? 0]));
}

function requireInsideDeadline(actual, expected) {
  if (!Number.isFinite(actual) || actual > expected) {
    throw new Error(`warm prefetch latency ${actual} exceeds ${expected}`);
  }
}
