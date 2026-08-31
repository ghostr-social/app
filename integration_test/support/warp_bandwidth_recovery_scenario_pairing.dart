part of 'warp_bandwidth_recovery_scenario.dart';

typedef _PairedBandwidthEvidence = ({
  WarpDecisionRecord decision,
  WarpPlanEvidence plan,
});

Future<WarpDecisionRecord> _waitForLossDecision(
  WidgetTester tester,
  _PacedFeed opened,
  int confirmedAtEpochMs,
) {
  final journey = opened.journey;
  return journey.waitForDecision(
    tester,
    (decision) => _isLossDecision(decision, opened, confirmedAtEpochMs),
    afterSequence: opened.baselineDecision.sequence,
  );
}

bool _isLossDecision(
  WarpDecisionRecord decision,
  _PacedFeed opened,
  int confirmedAtEpochMs,
) {
  final baseline = opened.baselineDecision;
  return decision.observedAtMs > confirmedAtEpochMs &&
      decision.networkThroughputBps < baseline.networkThroughputBps &&
      decision.appliesMeasuredNetworkRate &&
      decision.plannerNetworkRateBytesPerSecond! <
          baseline.plannerNetworkRateBytesPerSecond!;
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
