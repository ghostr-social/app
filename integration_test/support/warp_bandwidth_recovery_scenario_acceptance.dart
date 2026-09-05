part of 'warp_bandwidth_recovery_scenario.dart';

Future<void> _expectBandwidthAcceptance(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  await journey.telemetry.settled;
  await expectWarpRequestBounds(journey.evidence);
  final evaluation = (await journey.evidence.page()).evaluation;
  expect(evaluation.efficiency.totalBytes, greaterThan(0));
  expect(evaluation.efficiency.usefulWatchedBytes, greaterThan(0));
  expect(evaluation.efficiency.duplicateHedgeBytes, 0);
  expect(evaluation.semantics.transportSubstitutions, 0);
  // Remote handlers may still be acknowledging closed sockets.
  expect(
    journey.resources.origin.maximumConcurrentResponses,
    lessThanOrEqualTo(4),
  );
  expect(
    find.byType(VideoPlayer, skipOffstage: false).evaluate().length,
    inInclusiveRange(1, 2),
  );
  expect(journey.hadPlaybackError, isFalse);
  _expectOriginIntegrity(journey);
  await journey.reportSchedulingEvidence();
}

void _expectOriginIntegrity(WarpFeedPlaybackJourney journey) {
  final coverages = journey.resources.origin.bodyRequestedIds
      .map(journey.resources.origin.coverageFor)
      .toList();
  for (final coverage in coverages) {
    expect(coverage.isWithinObject, isTrue);
    expect(
      coverage.hasReplayIntegrityWithin(
        cancellationOverlapBudgetBytes: deviceCancellationWasteTargetBytes,
      ),
      isTrue,
    );
  }
  expect(
    progressiveReplayCancellationOverlapWithin(
      coverages,
      budgetBytes: deviceCancellationWasteTargetBytes,
    ),
    isTrue,
  );
}
