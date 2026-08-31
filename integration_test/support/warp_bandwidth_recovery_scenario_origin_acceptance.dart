part of 'warp_bandwidth_recovery_scenario.dart';

typedef _BandwidthAcceptanceEvidence = ({
  List<String> observedIds,
  WarpEvaluationSnapshot evaluation,
});

Future<_BandwidthAcceptanceEvidence> _waitForBandwidthEvidenceFence(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  for (var attempt = 0; attempt <= _fixtureIds.length; attempt += 1) {
    final requested = journey.resources.origin.bodyRequestedIds;
    expect(requested, isNotEmpty);
    await journey.waitForNativeStoreCoverage(tester, requested);
    await journey.waitForOriginQuiescence(tester, _fixtureIds);
    final settled = journey.resources.origin.bodyRequestedIds;
    if (!setEquals(requested.toSet(), settled.toSet())) continue;
    final beforePage = journey.originSnapshot(_fixtureIds);
    await journey.telemetry.settled;
    final evaluation = (await journey.evidence.page()).evaluation;
    final finalIds = journey.resources.origin.bodyRequestedIds;
    final stable =
        setEquals(settled.toSet(), finalIds.toSet()) &&
        mapEquals(beforePage, journey.originSnapshot(_fixtureIds));
    if (!stable || _hasServingFixtureRequest(journey)) continue;
    journey.expectNativeStoreCoverage(finalIds);
    return (observedIds: finalIds, evaluation: evaluation);
  }
  fail('Origin/store evidence did not reach a fixed point.');
}

bool _hasServingFixtureRequest(WarpFeedPlaybackJourney journey) {
  final paths = _fixtureIds.map((id) => '/$id.mp4').toSet();
  return journey.resources.origin.requests.any(
    (request) =>
        paths.contains(request.path) &&
        request.outcome == ProgressiveOriginRequestOutcome.serving,
  );
}

void _expectBandwidthOriginIntegrity(
  WarpFeedPlaybackJourney journey,
  List<String> observedIds,
) {
  final coverages = <ProgressiveOriginCoverage>[];
  for (final id in observedIds) {
    final coverage = journey.resources.origin.coverageFor(id);
    coverages.add(coverage);
    final evidence = _bandwidthOriginEvidence(journey, id, coverage);
    expect(coverage.isWithinObject, isTrue, reason: evidence);
    expect(
      coverage.hasReplayIntegrityWithin(
        cancellationOverlapBudgetBytes: deviceCancellationWasteTargetBytes,
      ),
      isTrue,
      reason: evidence,
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

String _bandwidthOriginEvidence(
  WarpFeedPlaybackJourney journey,
  String id,
  ProgressiveOriginCoverage coverage,
) {
  return '$id ranges=${coverage.servedRanges}; '
      'requests=${journey.originRequestEvidence([id])}; '
      'chunks=${journey.originChunkEvidence([id])}';
}
