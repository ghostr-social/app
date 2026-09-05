part of 'warp_stale_validator_rotation_scenario.dart';

extension _WarpValidatorRotationAssertions on _WarpValidatorRotationDriver {
  bool _firstBodiesRetired() {
    final held = fixture.requests.where(
      (item) =>
          item.generation == WarpValidatorGeneration.first && item.wasHeld,
    );
    return held.isNotEmpty && held.every((item) => item.isTerminal);
  }

  void _expectLiveContract(
    PlaybackFocus replacement,
    ({Duration before, Duration after}) advance,
    Uint8List bytes,
  ) {
    _expectRotationRequests();
    final session = graph.telemetry.probe.sessionFor(replacement)!;
    final presented = graph.telemetry.probe.presentationFor(replacement)!;
    final stage = graph.playerStages.forPresentation(
      session.deliveryId,
      presented.elapsed,
    );
    expect(stage?.firstFrameAt, isNotNull);
    expect(graph.telemetry.probe.playingLatency(replacement), isNotNull);
    expect(advance.after, greaterThan(advance.before));
    expect(sha256.convert(bytes), sha256.convert(fixture.secondBytes));
    expect(sha256.convert(bytes), isNot(sha256.convert(fixture.firstBytes)));
    expect(unavailableWasVisible, isFalse);
    expect(find.text('Video unavailable'), findsNothing);
  }

  void _expectRotationRequests() {
    final retired = fixture.requests.where(
      (item) =>
          item.wasHeld && item.generation == WarpValidatorGeneration.first,
    );
    expect(retired, isNotEmpty);
    expect(retired.every(_isRetiredFirstRequest), isTrue);
    final guarded = fixture.requests.where(
      (item) =>
          item.generation == WarpValidatorGeneration.second &&
          item.method == 'GET' &&
          item.range != null &&
          item.ifRange == fixture.firstValidator,
    );
    expect(guarded, isNotEmpty);
    expect(guarded.every((item) => item.statusCode == HttpStatus.ok), isTrue);
    expect(
      fixture.redirectTargets.map((target) => target.path).toSet(),
      containsAll(['/generation-a.mp4', '/generation-b.mp4']),
    );
  }

  Future<void> _teardown() async {
    await tester.pumpWidget(const SizedBox.shrink());
    await _wait(_isQuiescent, timeout: const Duration(seconds: 15));
  }

  bool _isQuiescent() {
    return _allAttempts.isNotEmpty &&
        _allAttempts.every((attempt) => attempt.isTerminal) &&
        fixture.activeRequestCount == 0 &&
        videoPlaybackCapacityOf(graph.playback).isQuiescent &&
        find.byType(VideoPlayer, skipOffstage: false).evaluate().isEmpty;
  }

  void _expectQuiescent() {
    expect(_isQuiescent(), isTrue);
    expect(peakMountedPlayers, lessThanOrEqualTo(2));
    expect(peakControllerCapacity, lessThanOrEqualTo(2));
    // A held old response remains in the fixture until it observes cancellation.
    expect(fixture.maximumConcurrentRequests, lessThanOrEqualTo(4));
    expect(fixture.totalRequestCount, lessThanOrEqualTo(20));
  }

  List<WarpFeedPlayerStageEvidence> get _allAttempts {
    final attempts = <WarpFeedPlayerStageEvidence>{};
    for (final event in scenario.events) {
      final delivery = graph.focus.deliveryForEvent(event.id);
      if (delivery != null) {
        attempts.addAll(graph.playerStages.attemptsFor(delivery));
      }
    }
    return attempts.toList();
  }
}

bool _isRetiredFirstRequest(WarpValidatorRequest item) {
  return item.outcome == WarpValidatorRequestOutcome.retiredAfterRotation ||
      item.outcome == WarpValidatorRequestOutcome.clientCanceled;
}
