import {IMPAIRMENT_SCENARIOS} from "./impairment_scenarios.mjs";

const RELEASED_STORAGE_BYTES = 64 * 1_024 * 1_024;
const DEFAULT_OBSERVATION_SECONDS = 0.75;
const PACKET_LOSS_OBSERVATION_SECONDS = 2.5;
const ACTION_BUILDERS = Object.freeze({
  network: networkActions,
  focus: focusActions,
  storage: storageSteps,
  origin: () => [],
});

export function impairmentOriginOptions(name) {
  const origin = definition(name)?.origin;
  if (!origin) return {};
  if (origin.abort_first_attempts) {
    return {
      abortFirstAttempts: origin.abort_first_attempts,
      abortAfterBytes: origin.abort_after_bytes,
    };
  }
  return origin.fail_source ? {failSource: origin.fail_source} : {};
}

export function impairmentVideoRegistration(name, video, origin) {
  if (name !== "source_failure") return {url: `${origin}/${video}.mp4`, mirrors: []};
  return {
    url: `${origin}/${video}-primary.mp4`,
    mirrors: [`${origin}/${video}-mirror.mp4`],
  };
}

export function impairmentActions(name) {
  const scenario = definition(name);
  if (!scenario) return [];
  const kind = Object.keys(scenario)[0];
  return ACTION_BUILDERS[kind]?.(scenario[kind]) ?? [];
}

export function bootstrapImpairmentActions(name) {
  return impairmentActions(name).filter(isBootstrapControl);
}

export function playbackImpairmentActions(name) {
  return impairmentActions(name).filter((action) => !isBootstrapControl(action));
}

export function playbackObservationSeconds(name, index) {
  if (name === "packet_loss" && index === 2) return PACKET_LOSS_OBSERVATION_SECONDS;
  return DEFAULT_OBSERVATION_SECONDS;
}

function isBootstrapControl(action) {
  return action.at_ms === 0 && action.kind !== "focus";
}

function networkStep(step) {
  return {
    at_ms: step.at_ms,
    kind: "network",
    payload: {
      bandwidth_kbps: step.bandwidth_kbps,
      latency_ms: step.latency_ms ?? 0,
      max_connections_per_host: step.max_connections_per_host ?? 3,
    },
  };
}

function networkActions(network) {
  const steps = network.steps ?? [{at_ms: 0, ...network}];
  return steps.map(networkStep);
}

function focusActions(focus) {
  return focus.map((step, index, all) => focusStep(step, index < all.length - 1));
}

function focusStep(step, superseded) {
  return {
    at_ms: step.at_ms,
    kind: "focus",
    payload: {index: step.index, superseded},
  };
}

function storageSteps(storage) {
  return [
    {at_ms: 0, kind: "storage", payload: {budget_bytes: storage.budget_bytes}},
    {
      at_ms: storage.release_at_ms,
      kind: "storage",
      payload: {budget_bytes: RELEASED_STORAGE_BYTES},
    },
  ];
}

function definition(name) {
  return name ? IMPAIRMENT_SCENARIOS[name] : null;
}
