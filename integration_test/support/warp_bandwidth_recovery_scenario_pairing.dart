part of 'warp_bandwidth_recovery_scenario.dart';

typedef _PairedBandwidthEvidence = ({
  WarpDecisionRecord decision,
  WarpPlanEvidence plan,
});

Future<_PairedBandwidthEvidence> _waitForLossPair(
  WidgetTester tester,
  _PacedFeed opened,
  int confirmedAtEpochMs,
) async {
  final journey = opened.journey;
  final retained = await journey.evidence.page();
  final pair = await journey.waitForDecisionPlanPair(
    tester,
    (decision, plan) => _isLossPair(decision, plan, opened, confirmedAtEpochMs),
    afterSequence: opened.baselineDecision.sequence,
    afterRevision: retained.planPage.beforeOldestRetainedRevision,
  );
  journey.reportPlan(pair.plan);
  return pair;
}

bool _isLossPair(
  WarpDecisionRecord decision,
  WarpPlanEvidence plan,
  _PacedFeed opened,
  int confirmedAtEpochMs,
) {
  final baseline = opened.baselineDecision;
  return decision.observedAtMs > confirmedAtEpochMs &&
      decision.networkThroughputBps < baseline.networkThroughputBps &&
      decision.appliesMeasuredNetworkRate &&
      decision.plannerNetworkRateBytesPerSecond! <
          baseline.plannerNetworkRateBytesPerSecond! &&
      plan.networkStatusGeneration == 1 &&
      plan.networkClass == WarpNetworkClass.wifi &&
      plan.plan.workBreadth > 0 &&
      plan.coversFocusGeneration(opened.focusGeneration);
}

void _reportNetworkResponse(
  String phase,
  WarpDecisionRecord decision,
  int confirmedAtEpochMs,
) {
  final latencyMs = decision.observedAtMs - confirmedAtEpochMs;
  expect(latencyMs, greaterThan(0));
  debugPrint(
    'WARP_LINK $phase response_ms=$latencyMs '
    'throughput_bps=${decision.networkThroughputBps} '
    'planner_Bps=${decision.plannerNetworkRateBytesPerSecond}',
  );
}
