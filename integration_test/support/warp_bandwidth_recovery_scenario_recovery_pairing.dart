part of 'warp_bandwidth_recovery_scenario.dart';

typedef _RecoveryBaselineQuery = ({PlaybackFocus focus, int afterRevision});

Future<_RecoveryFocus> _waitForRecoveryBaseline(
  WidgetTester tester,
  _PacedFeed opened,
  _ImpairedFeed impaired,
  _RecoveryBaselineQuery query,
) async {
  final journey = opened.journey;
  final pair = await journey.waitForDecisionPlanPair(
    tester,
    (decision, plan) =>
        _isRecoveryBaselinePlan(plan, journey, query) &&
        _isRecoveryBaselineDecision(decision, opened, plan),
    afterSequence: impaired.decision.sequence,
    afterRevision: query.afterRevision,
  );
  final ready = await _waitForRecoveryReady(tester, journey, query.focus);
  final frontier = _recoveryFrontier(journey, ready);
  final trigger = journey.resources.origin
      .armBandwidthChangeAfterNextConfirmedChunk(
        frontier.transitionPaths,
        bandwidthKbps: 2500,
      );
  journey.reportPlan(pair.plan);
  return (
    focus: query.focus,
    decision: pair.decision,
    ready: ready,
    frontier: frontier,
    recoveryTrigger: trigger,
  );
}

WarpRecoveryFrontier _recoveryFrontier(
  WarpFeedPlaybackJourney journey,
  WarpReadyWindow ready,
) {
  final snapshot = ready.snapshot;
  final reserve = ready.plan.plan.readyReserve;
  final future = journey.allFutureRemotePaths();
  return warpRecoveryFrontier((
    futureRosterPaths: future,
    candidatePaths: future.take(reserve.candidateCount).toList(),
    projectedPaths: journey.remotePathsFor(snapshot.upcoming),
    candidateStates: reserve.candidateStates,
    orderedReady: reserve.orderedReady,
    candidateCount: reserve.candidateCount,
    minimumReadyDepth: 2,
  ));
}

Future<WarpReadyWindow> _waitForRecoveryReady(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  PlaybackFocus focus,
) {
  final generation = journey.focus.generationFor(focus)!;
  final deliveryId = journey.telemetry.probe.sessionFor(focus)!.deliveryId;
  return journey.waitForRecoveryFrontierWindow(
    tester,
    generation,
    currentDeliveryId: deliveryId,
    minimumDepth: 2,
    afterSequence: focus.sequence,
  );
}

bool _isRecoveryBaselinePlan(
  WarpPlanEvidence item,
  WarpFeedPlaybackJourney journey,
  _RecoveryBaselineQuery query,
) {
  return item.networkStatusGeneration == 1 &&
      item.networkClass == WarpNetworkClass.wifi &&
      item.plan.workBreadth > 0 &&
      item.coversFocusGeneration(journey.focus.generationFor(query.focus)!);
}

bool _isRecoveryBaselineDecision(
  WarpDecisionRecord item,
  _PacedFeed opened,
  WarpPlanEvidence plan,
) {
  final baseline = opened.baselineDecision;
  return item.observedAtMs == plan.observedAtMs &&
      item.networkThroughputBps < baseline.networkThroughputBps &&
      item.appliesMeasuredNetworkRate &&
      item.plannerNetworkRateBytesPerSecond! <
          baseline.plannerNetworkRateBytesPerSecond!;
}
