import {ArtifactStore} from "./artifacts.mjs";
import {createEvidenceSender} from "./impairment_evidence.mjs";
import {
  bootstrapImpairmentActions, impairmentOriginOptions,
} from "./impairment_plan.mjs";
import {validateJourney} from "./journey_outcome.mjs";
import {OwnedLifecycle} from "./lifecycle.mjs";
import {measureQoe, requireQoeTargets} from "./qoe_metrics.mjs";
import {
  measureOrderedPrefetch, requireOrderedPrefetchTargets,
} from "./ordered_prefetch_acceptance.mjs";
import {ORDERED_PREFETCH_TARGETS, QOE_TARGETS} from "./qoe_targets.mjs";
import {establishOrderedFocus} from "./ordered_focus_warmup.mjs";
import {recordInitialFocusLocality} from "./focus_locality.mjs";
import {createRunnerBoundaries} from "./runner_boundaries.mjs";
import {playRunnerJourney} from "./runner_journey.mjs";
import {waitForWarmPrefetch} from "./warm_prefetch.mjs";
import {poll, withDeadline} from "./wait.mjs";

const TOTAL_TIMEOUT_MS = 180_000;
const ORDERED_PREFETCH_SCENARIO = "ordered_prefetch";
const ORDERED_PREFETCH_OBSERVATION_MS = 500;

export function createVideoUserE2eRunner(overrides = {}) {
  const boundaries = createRunnerBoundaries(overrides);
  return (input) => runVideoUserE2eWith(boundaries, input);
}

export const runVideoUserE2e = createVideoUserE2eRunner();

async function runVideoUserE2eWith(boundaries, {root, environment, browser, scenario = null}) {
  const files = await boundaries.createRunFiles(root, environment);
  const context = createContext({files, environment, browser, scenario, boundaries});
  try {
    const trace = await withDeadline({
      run: (signal) => runScenario(context, signal),
      timeoutMs: TOTAL_TIMEOUT_MS,
      label: "local video user E2E",
    });
    context.trace = trace;
    await boundaries.writeSuccess(context, trace);
    return {artifacts: files.artifacts, trace};
  } catch (error) {
    await boundaries.writeFailure(context, error);
    throw new Error(`${error.message}; artifacts: ${files.artifacts}`, {cause: error});
  } finally {
    context.browserRun?.cdp.close();
    await context.origin?.close();
    await context.lifecycle.teardown();
    await boundaries.removeTransientRunFiles(files);
  }
}

async function runScenario(context, signal) {
  const admission = await prepareAdmission(context, signal);
  const session = await prepareBrowser(context, admission, signal);
  if (isOrderedPrefetch(context)) await observeOrderedPrefetch(context, session, signal);
  else await playRunnerJourney(context, admission.ids, session.trace, signal);
  return finishTrace(context, session.trace);
}

async function observeOrderedPrefetch(context, session, signal) {
  await session.warm;
  await context.boundaries.delay(ORDERED_PREFETCH_OBSERVATION_MS, signal);
  session.trace.warm_prefetch.post_readiness_observation_ms = ORDERED_PREFETCH_OBSERVATION_MS;
}

async function prepareAdmission(context, signal) {
  context.origin = await context.boundaries.startLocalOrigin(
    impairmentOriginOptions(context.scenario),
  );
  context.server = await context.boundaries.startServer({...context, signal, timeoutMs: 90_000});
  await applyBootstrapImpairments(context);
  const ids = await context.boundaries.registerOrderedVideos({
    server: context.server.url,
    origin: context.origin.url,
    scenario: context.scenario,
  });
  return {ids};
}

async function prepareBrowser(context, admission, signal) {
  context.browserRun = await context.boundaries.startBrowser({
    ...context, signal, url: context.server.url, timeoutMs: 30_000,
  });
  const trace = createTrace(context.scenario, admission.ids, context.impairments);
  context.trace = trace;
  const focus = await establishOrderedFocus({
    ids: admission.ids,
    read: () => context.boundaries.refreshDebugSnapshot(context.browserRun.page),
    select: (id) => context.boundaries.selectVideoFocus(context.server.url, id),
    record: initialFocusRecorder(context, admission.ids, trace),
    warm: isOrderedPrefetch(context)
      ? (timing) => warmPrefetch(context, admission.ids, timing, signal)
      : undefined,
  });
  await waitForVideos(context, admission.ids, signal);
  await context.boundaries.requireUserStartsPlayback(context.browserRun.page);
  trace.started_at_epoch_ms = focus.startedAt;
  return {trace, warm: focus.warm};
}

function finishTrace(context, trace) {
  trace.requests = context.browserRun.ledger.entries;
  trace.origin_requests = structuredClone(context.origin.requests);
  if (isOrderedPrefetch(context)) return finishOrderedPrefetch(trace);
  trace.qoe = measureQoe(trace);
  validateJourney(trace);
  requireQoeTargets(trace.qoe, QOE_TARGETS);
  return trace;
}

function finishOrderedPrefetch(trace) {
  trace.qoe = measureOrderedPrefetch(trace);
  requireOrderedPrefetchTargets(trace.qoe, ORDERED_PREFETCH_TARGETS);
  return trace;
}

function createTrace(scenario, ids, impairments) {
  return {
    scenario,
    started_at_epoch_ms: null,
    video_ids: Object.fromEntries(ids.map((id, index) => [`v${index}`, id])),
    ordered_video_ids: ids,
    clicks: [],
    samples: [],
    requests: [],
    impairments,
  };
}

async function warmPrefetch(context, ids, timing, signal) {
  return waitForWarmPrefetch({
    orderedIds: ids,
    protectedCount: ORDERED_PREFETCH_TARGETS.protected_count,
    baseline: timing.baseline,
    minimumBytes: ORDERED_PREFETCH_TARGETS.minimum_bytes,
    deadlineMs: ORDERED_PREFETCH_TARGETS.latency_ms,
    startedAt: timing.startedAt,
    read: () => context.boundaries.refreshDebugSnapshot(context.browserRun.page),
    signal,
    onEvidence: (evidence) => { context.trace.warm_prefetch = evidence; },
  });
}

function initialFocusRecorder(context, ids, trace) {
  return ({id, baseline, startedAt}) => recordInitialFocusLocality({
    trace, id, state: baseline, startedAt, orderedIds: ids,
    protectedCount: ORDERED_PREFETCH_TARGETS.protected_count,
    minimumBytes: ORDERED_PREFETCH_TARGETS.minimum_bytes,
    originRequests: context.origin.requests,
  });
}

function isOrderedPrefetch(context) {
  return context.scenario === ORDERED_PREFETCH_SCENARIO;
}

async function applyBootstrapImpairments(context) {
  const send = createEvidenceSender({evidence: context.impairments,
    send: (action) => context.boundaries.sendControlAction(context.server.url, action)});
  for (const action of bootstrapImpairmentActions(context.scenario)) {
    await send(action);
  }
}

async function waitForVideos(context, ids, signal) {
  await poll({
    read: () => context.boundaries.refreshDebugSnapshot(context.browserRun.page),
    accept: (state) => ids.every((id) => state?.videos.some((video) => video.id === id)),
    timeoutMs: 15_000,
    intervalMs: 100,
    label: "local videos in debug state",
    signal,
  });
}

function createContext(input) {
  return {
    files: input.files,
    environment: input.environment,
    browser: input.browser,
    scenario: input.scenario,
    boundaries: input.boundaries,
    lifecycle: new OwnedLifecycle(),
    store: new ArtifactStore({directory: input.files.artifacts}),
    origin: null,
    server: null,
    browserRun: null,
    trace: null,
    impairments: [],
  };
}
