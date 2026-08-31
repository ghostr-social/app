part of 'warp_bandwidth_recovery_scenario.dart';

typedef _RecoveryResponseQuery = ({
  _RecoveryFocus recovery,
  PlaybackFocus focus,
  int confirmedAtEpochMs,
});

Future<_PairedBandwidthEvidence> _waitForRecoveryPair(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  _RecoveryResponseQuery query,
) async {
  final pair = await journey.waitForDecisionPlanPair(
    tester,
    (decision, plan) => _isRecoveryPair(decision, plan, journey, query),
    afterSequence: query.recovery.decision.sequence,
    afterRevision: query.recovery.planRevision,
  );
  journey.reportPlan(pair.plan);
  return pair;
}

bool _isRecoveryPair(
  WarpDecisionRecord decision,
  WarpPlanEvidence plan,
  WarpFeedPlaybackJourney journey,
  _RecoveryResponseQuery query,
) {
  final baseline = query.recovery.decision;
  return decision.observedAtMs > query.confirmedAtEpochMs &&
      decision.networkThroughputBps > baseline.networkThroughputBps &&
      decision.appliesMeasuredNetworkRate &&
      decision.plannerNetworkRateBytesPerSecond! >
          baseline.plannerNetworkRateBytesPerSecond! &&
      plan.networkStatusGeneration == 1 &&
      plan.networkClass == WarpNetworkClass.wifi &&
      plan.plan.workBreadth > 0 &&
      plan.coversFocusGeneration(journey.focus.generationFor(query.focus)!);
}
