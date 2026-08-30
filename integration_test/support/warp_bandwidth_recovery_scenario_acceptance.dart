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

Future<void> _verifyBandwidthAcceptance(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  await journey.telemetry.settled;
  await journey.waitForOriginQuiescence(tester, _fixtureIds);
  final observedIds = journey.resources.origin.bodyRequestedIds;
  expect(observedIds, isNotEmpty);
  for (final id in observedIds) {
    final coverage = journey.resources.origin.coverageFor(id);
    final evidence =
        '$id ranges=${coverage.servedRanges}; '
        'requests=${journey.originRequestEvidence([id])}; '
        'chunks=${journey.originChunkEvidence([id])}';
    expect(coverage.isWithinObject, isTrue, reason: evidence);
    expect(coverage.duplicateBytes, 0, reason: evidence);
  }
  await journey.waitForReplayStoreCoverage(tester, observedIds);
  final page = await journey.evidence.page();
  final evaluation = page.evaluation;
  expect(evaluation.userVisible.startupFailures, 0);
  expect(evaluation.userVisible.stallRatioBps, lessThanOrEqualTo(100));
  expect(evaluation.efficiency.totalBytes, greaterThan(0));
  expect(evaluation.efficiency.usefulWatchedBytes, greaterThan(0));
  expect(
    evaluation.efficiency.abortedBytes,
    lessThanOrEqualTo(deviceCancellationWasteTargetBytes),
  );
  expect(evaluation.efficiency.duplicateHedgeBytes, 0);
  expect(evaluation.efficiency.requestCount, greaterThan(0));
  expect(evaluation.budget.instantaneousViolations, 0);
  expect(evaluation.readiness.readyCoverageMs, greaterThan(0));
  expect(evaluation.semantics.transportSubstitutions, 0);
  expect(journey.hadPlaybackError, isFalse);
  expect(journey.focus.hadTransportRescue, isFalse);
  debugPrint(
    'WARP_BANDWIDTH total=${evaluation.efficiency.totalBytes} '
    'useful=${evaluation.efficiency.usefulWatchedBytes} '
    'aborted=${evaluation.efficiency.abortedBytes} '
    'stall_bps=${evaluation.userVisible.stallRatioBps}',
  );
}
