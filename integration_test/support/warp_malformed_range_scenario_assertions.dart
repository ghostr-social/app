part of 'warp_malformed_range_scenario.dart';

extension _MalformedRangeAssertions on _MalformedRangeScenario {
  List<VideoDeliverySnapshot> get _malformedSnapshots => journey
      .graph
      .deliveryProbe
      .observations
      .map((observation) => observation.snapshot)
      .where((snapshot) => snapshot.deliveryId == malformedId)
      .toList();

  void expectRejectedRangeIsNotReady() {
    final snapshots = _malformedSnapshots;
    expect(snapshots, isNotEmpty);
    expect(
      snapshots.every((snapshot) => snapshot.bytesPresent == BigInt.zero),
      isTrue,
    );
    expect(
      snapshots.any(
        (snapshot) => snapshot.phase == VideoDeliveryPhase.startable,
      ),
      isFalse,
    );
    expect(_malformedPreparationWasStartable(), isFalse);
    expect(journey.playerStages.attemptsFor(malformedId), isEmpty);
  }

  void expectBoundedOriginWork() {
    final bodies = journey.resources.origin
        .requestsFor('next')
        .where((request) => request.method == 'GET')
        .toList();
    final evidence = journey.originRequestEvidence(['next']);
    expect(bodies, isNotEmpty, reason: evidence);
    expect(bodies.length, lessThanOrEqualTo(4), reason: evidence);
    expect(
      bodies.every(
        (body) => body.outcome != ProgressiveOriginRequestOutcome.serving,
      ),
      isTrue,
      reason: evidence,
    );
    final served = bodies.fold(0, (sum, body) => sum + body.servedBytes);
    expect(
      served,
      lessThanOrEqualTo(journey.resources.origin.objectLength * 4),
      reason: evidence,
    );
  }

  bool _malformedPreparationWasStartable() {
    return journey.preparation.observations.any((observation) {
      return observation.upcoming.any((asset) {
        return asset.authority.deliveryId == malformedId &&
            asset.readiness.isStructurallyStartable;
      });
    });
  }

  Future<void> expectDecodedRescue(
    WidgetTester tester,
    PlaybackFocus focus,
  ) async {
    expect(focus.cause, FeedFocusCause.transportRescue);
    expect(focus.rescue, isNotNull);
    expect(focus.rescue?.rankDisplacement, 1);
    journey.expectSinglePlayerAttempt(focus);
    expect(find.text('Video unavailable'), findsNothing);
    final before = journey.telemetry.probe.latestPositionFor(focus)!;
    await journey.pumpFor(tester, const Duration(seconds: 1));
    expect(
      journey.telemetry.probe.latestPositionFor(focus),
      greaterThan(before),
    );
    _reportEvidence(focus);
  }

  void _reportEvidence(PlaybackFocus focus) {
    final requests = journey.resources.origin.requestsFor('next');
    final bytes = requests.fold(0, (sum, request) => sum + request.servedBytes);
    debugPrint(
      'WARP_MALFORMED_RANGE requests=${requests.length} bytes=$bytes '
      'snapshots=${_malformedSnapshots.length} '
      'rescue=${focus.rescue?.reason.name}:1',
    );
  }
}
