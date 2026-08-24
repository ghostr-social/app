import {duplicateCompletedOriginBytes} from "./duplicate_origin_metrics.mjs";
import {protectedTransitionLatency} from "./transition_metrics.mjs";
export {requireQoeTargets} from "./qoe_target_validation.mjs";

const PLAYING = "playing";
const STALLED = new Set(["buffering", "stalled"]);

export function measureQoe(trace) {
  const clicks = (trace.clicks ?? []).filter((click) => !click.superseded);
  const latency = measureLatencies(clicks, trace.samples ?? []);
  const rebuffer = measureRebuffer(clicks, trace.samples ?? []);
  return {
    ...latency,
    ...rebuffer,
    cancellation_waste_bytes: cancellationWaste(trace),
    ahead_prefetch_bytes: aheadPrefetch(trace, clicks, trace.samples ?? []),
    duplicate_completed_origin_bytes: duplicateCompletedOriginBytes(trace.origin_requests),
    protected_transition_latency_ms: protectedTransitionLatency(
      trace.clicks, trace.samples,
    ),
  };
}

function measureLatencies(clicks, samples) {
  const values = clicks.map((click) => playingLatency(click, samples));
  return {
    startup_latency_ms: values[0] ?? Number.POSITIVE_INFINITY,
    focus_switch_latency_ms: values.length < 2 ? 0 : Math.max(...values.slice(1)),
    focus_switch_latencies_ms: values.slice(1),
  };
}

function playingLatency(click, samples) {
  const sample = samples.find((entry) => {
    return entry.at_ms >= click.at_ms
      && entry.player?.id === click.id
      && entry.player?.phase === PLAYING;
  });
  return sample ? sample.at_ms - click.at_ms : Number.POSITIVE_INFINITY;
}

function measureRebuffer(clicks, samples) {
  const totals = clicks.map((click, index) => {
    const end = clicks[index + 1]?.at_ms ?? Number.POSITIVE_INFINITY;
    return segmentRebuffer(click, end, samples);
  });
  const observed = totals.reduce((sum, value) => sum + value.observed, 0);
  const stalled = totals.reduce((sum, value) => sum + value.stalled, 0);
  return {
    rebuffer_duration_ms: stalled,
    observed_playback_ms: observed,
    rebuffer_ratio: observed === 0 ? Number.POSITIVE_INFINITY : stalled / observed,
  };
}

function segmentRebuffer(click, end, samples) {
  const segment = samples.filter((sample) => {
    return sample.at_ms >= click.at_ms && sample.at_ms <= end && sample.player?.id === click.id;
  });
  const started = segment.findIndex((sample) => sample.player.phase === PLAYING);
  if (started < 0) return {observed: 0, stalled: 0};
  return intervalTotals(segment.slice(started));
}

function intervalTotals(samples) {
  let observed = 0;
  let stalled = 0;
  for (let index = 0; index < samples.length - 1; index += 1) {
    const duration = Math.max(0, samples[index + 1].at_ms - samples[index].at_ms);
    observed += duration;
    if (STALLED.has(samples[index].player.phase)) stalled += duration;
  }
  return {observed, stalled};
}

function cancellationWaste(trace) {
  return (trace.origin_requests ?? []).reduce((sum, request) => {
    if (!request.canceled || request.completed) return sum;
    if (!request.chunk_events || !requestVideoId(trace, request)) {
      return sum + (request.bytes_sent ?? 0);
    }
    return sum + request.chunk_events.reduce((bytes, event) => {
      return chunkWasSentAfterFocusLeft(trace, request, event) ? bytes + event.bytes : bytes;
    }, 0);
  }, 0);
}

function chunkWasSentAfterFocusLeft(trace, request, event) {
  const elapsed = event.at_ms - trace.started_at_epoch_ms;
  const click = (trace.clicks ?? []).findLast((entry) => entry.at_ms <= elapsed);
  return click && click.id !== requestVideoId(trace, request);
}

function requestVideoId(trace, request) {
  return trace.video_ids?.[request.video];
}

function aheadPrefetch(trace, clicks, samples) {
  return playbackAheadPrefetch(trace, clicks, samples);
}

function playbackAheadPrefetch(trace, clicks, samples) {
  const click = clicks[0];
  if (!click) return 0;
  const end = clicks[1]?.at_ms ?? Number.POSITIVE_INFINITY;
  const baseline = {};
  return Math.max(0, ...samples.map((sample) => {
    if (!isInitialFocusSample(sample, click, end)) return 0;
    return sample.state.videos.reduce((sum, video) => {
      return video.id === click.id ? sum : sum + prefetchGain(video, baseline);
    }, 0);
  }));
}

function prefetchGain(video, baseline) {
  return Math.max(0, video.downloaded_bytes - (baseline[video.id] ?? 0));
}

function isInitialFocusSample(sample, click, end) {
  if (sample.at_ms < click.at_ms || sample.at_ms >= end || sample.player?.id !== click.id) {
    return false;
  }
  const current = sample.state?.videos?.find((video) => video.id === click.id);
  return current && current.downloaded_bytes < current.total_bytes;
}
