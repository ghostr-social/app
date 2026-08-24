import {
  controlPoint, debugSnapshot, dispatchTrustedClick, playerSnapshot,
} from "./page_runtime.mjs";
import {compactDebugState} from "./debug_state_evidence.mjs";
import {poll} from "./wait.mjs";

const PLAYING_POLL_MS = 25;
const PROGRESS_POLL_MS = 100;

export async function requireUserStartsPlayback(page) {
  const player = await playerSnapshot(page);
  if (player.id !== null || !player.paused) {
    throw new Error("playback started before a visible user control was clicked");
  }
}

export async function clickVideo(page, id, signal) {
  const point = await poll({
    read: () => controlPoint(page, id),
    accept: (value) => value?.ready === true,
    timeoutMs: 10_000,
    intervalMs: 100,
    label: `${id} visible play control`,
    signal,
  });
  if (!point.label?.startsWith("Play ")) throw new Error("untrusted play control label");
  await dispatchTrustedClick(page, point);
}

export async function watchProgress(input) {
  const first = await watchUntilPlaying(input);
  await collectUntil({
    ...input,
    accept: progressed(input.id, first.player.current_time, input.observedSeconds),
    label: "media time",
  });
}

export function watchUntilPlaying(input) {
  return collectUntil(
    {...input, accept: isPlaying(input.id), label: "playing"},
    PLAYING_POLL_MS,
  );
}

function collectUntil(input, intervalMs = PROGRESS_POLL_MS) {
  return poll({
    read: () => captureSample(input),
    accept: input.accept,
    timeoutMs: 15_000,
    intervalMs,
    label: `${input.id} ${input.label}`,
    signal: input.signal,
  });
}

async function captureSample(input) {
  const state = await debugSnapshot(input.page);
  const sample = {
    at_ms: Date.now() - input.started,
    player: await playerSnapshot(input.page),
    state: compactDebugState(state),
  };
  input.trace.samples.push(sample);
  return sample;
}

function isPlaying(id) {
  return (sample) => sample.player.id === id && sample.player.phase === "playing";
}

function progressed(id, first, observedSeconds) {
  return (sample) => isPlaying(id)(sample)
    && sample.player.current_time >= first + observedSeconds;
}
