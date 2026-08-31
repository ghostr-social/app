part of 'warp_bandwidth_recovery_scenario.dart';

const _minimumBaselineThroughputBps = 1000000;

Future<_PairedBandwidthEvidence> _waitForBaselinePair(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  int confirmedAtEpochMs,
  BigInt focusGeneration,
) {
  return journey.waitForDecisionPlanPair(
    tester,
    (decision, plan) =>
        _isBaselinePair(decision, plan, confirmedAtEpochMs, focusGeneration),
  );
}

bool _isBaselinePair(
  WarpDecisionRecord decision,
  WarpPlanEvidence plan,
  int confirmedAtEpochMs,
  BigInt focusGeneration,
) {
  return decision.observedAtMs > confirmedAtEpochMs &&
      decision.networkThroughputBps >= _minimumBaselineThroughputBps &&
      decision.appliesMeasuredNetworkRate &&
      plan.networkStatusGeneration == 1 &&
      plan.networkClass == WarpNetworkClass.wifi &&
      plan.coversFocusGeneration(focusGeneration) &&
      plan.plan.workBreadth >= 2;
}

void _reportBaseline(WarpDecisionRecord decision) {
  debugPrint(
    'WARP_LINK baseline at=${decision.observedAtMs} '
    'throughput_bps=${decision.networkThroughputBps} '
    'planner_Bps=${decision.plannerNetworkRateBytesPerSecond}',
  );
}
