import {createEvidenceSender} from "./impairment_evidence.mjs";
import {runImpairmentJourney} from "./impairment_journey.mjs";
import {
  playbackImpairmentActions, playbackObservationSeconds,
} from "./impairment_plan.mjs";

export function playRunnerJourney(context, ids, trace, signal) {
  return runImpairmentJourney(journeyInputs(context, ids, trace, signal));
}

function journeyInputs(context, ids, trace, signal) {
  const started = trace.started_at_epoch_ms;
  return {
    actions: playbackImpairmentActions(context.scenario),
    scenario: context.scenario,
    ids,
    trace,
    startedAt: started,
    now: Date.now,
    signal,
    click: (id) => context.boundaries.clickVideo(context.browserRun.page, id, signal),
    ...playbackWatchers(context, ids, trace, signal),
    send: evidenceSender(context, trace, started),
  };
}

function playbackWatchers(context, ids, trace, signal) {
  const started = trace.started_at_epoch_ms;
  return {
    watch: (id) => context.boundaries.watchProgress({
      page: context.browserRun.page,
      id,
      trace,
      started,
      signal,
      observedSeconds: playbackObservationSeconds(context.scenario, ids.indexOf(id)),
    }),
    watchStart: (id) => context.boundaries.watchUntilPlaying({
      page: context.browserRun.page, id, trace, started, signal,
    }),
  };
}

function evidenceSender(context, trace, startedAt) {
  return createEvidenceSender({
    evidence: trace.impairments,
    startedAt,
    read: () => context.boundaries.refreshDebugSnapshot(context.browserRun.page),
    send: (action) => context.boundaries.sendControlAction(context.server.url, action),
  });
}
