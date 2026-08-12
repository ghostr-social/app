import {
  controlPoint, debugSnapshot, dispatchTrustedClick, playerSnapshot,
} from "./page_runtime.mjs";
import {poll} from "./wait.mjs";

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
  return collectUntil({...input, accept: isPlaying(input.id), label: "playing"});
}

function collectUntil(input) {
  return poll({
    read: () => captureSample(input),
    accept: input.accept,
    timeoutMs: 15_000,
    intervalMs: 100,
    label: `${input.id} ${input.label}`,
    signal: input.signal,
  });
}

async function captureSample(input) {
  const sample = {
    at_ms: Date.now() - input.started,
    player: await playerSnapshot(input.page),
    state: await debugSnapshot(input.page),
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
