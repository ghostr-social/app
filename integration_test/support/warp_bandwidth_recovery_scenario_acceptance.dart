part of 'warp_bandwidth_recovery_scenario.dart';

const _fixtureIds = [
  'current',
  'next',
  'third',
  'fourth',
  'fifth',
  'sixth',
  'seventh',
  'eighth',
  'ninth',
  'tenth',
];
const _aggressiveConnectionCeiling = 4;

Future<void> _verifyBandwidthAcceptance(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  await journey.telemetry.settled;
  final evidence = await _waitForBandwidthEvidenceFence(tester, journey);
  _expectBandwidthOriginIntegrity(journey, evidence.observedIds);
  _expectBandwidthEvaluation(journey, evidence.evaluation);
  _reportBandwidthEvaluation(journey, evidence.evaluation);
}

void _expectBandwidthEvaluation(
  WarpFeedPlaybackJourney journey,
  WarpEvaluationSnapshot evaluation,
) {
  // TODO: Split focus supersession and calibrate aggregate QoE before gating it.
  // TODO: Rebuild WARP budget telemetry from admitted work before gating it.
  expect(evaluation.efficiency.totalBytes, greaterThan(0));
  expect(evaluation.efficiency.usefulWatchedBytes, greaterThan(0));
  expect(evaluation.efficiency.duplicateHedgeBytes, 0);
  expect(evaluation.efficiency.requestCount, greaterThan(0));
  expect(
    journey.resources.origin.maximumConcurrentResponses,
    lessThanOrEqualTo(_aggressiveConnectionCeiling),
  );
  expect(evaluation.readiness.readyCoverageMs, greaterThan(0));
  expect(evaluation.semantics.transportSubstitutions, 0);
  expect(journey.hadPlaybackError, isFalse);
  expect(journey.focus.hadTransportRescue, isFalse);
}

void _reportBandwidthEvaluation(
  WarpFeedPlaybackJourney journey,
  WarpEvaluationSnapshot evaluation,
) {
  debugPrint(
    'WARP_BANDWIDTH total=${evaluation.efficiency.totalBytes} '
    'useful=${evaluation.efficiency.usefulWatchedBytes} '
    'aborted=${evaluation.efficiency.abortedBytes} '
    'origin_peak=${journey.resources.origin.maximumConcurrentResponses} '
    'projected_budget_violations='
    '${evaluation.budget.instantaneousViolations} '
    'startup_failures=${evaluation.userVisible.startupFailures} '
    'stall_events=${evaluation.userVisible.stallEvents} '
    'stall_ms=${evaluation.userVisible.stallMs} '
    'stall_bps=${evaluation.userVisible.stallRatioBps}',
  );
}
