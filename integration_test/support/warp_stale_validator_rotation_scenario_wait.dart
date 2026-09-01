part of 'warp_stale_validator_rotation_scenario.dart';

extension _WarpValidatorRotationWait on _WarpValidatorRotationDriver {
  Future<PlaybackFocus> _waitForFocus(
    int index, {
    int afterSequence = 0,
  }) async {
    PlaybackFocus? focus;
    await _wait(() {
      focus = graph.focus.occurrenceAfter(
        scenario.events[index].id,
        afterSequence,
      );
      return focus != null;
    });
    return focus!;
  }

  Future<void> _waitForDecoded(PlaybackFocus focus) async {
    await _wait(() {
      final probe = graph.telemetry.probe;
      final presentation = probe.presentationFor(focus);
      final session = probe.sessionFor(focus);
      if (presentation == null || session == null) return false;
      final stage = graph.playerStages.forPresentation(
        session.deliveryId,
        presentation.elapsed,
      );
      return stage?.firstFrameAt != null && probe.playingLatency(focus) != null;
    });
  }

  Future<({Duration before, Duration after})> _waitForAdvancement(
    PlaybackFocus focus,
  ) async {
    final probe = graph.telemetry.probe;
    final before = probe.latestPositionFor(focus) ?? Duration.zero;
    await _wait(
      () => (probe.latestPositionFor(focus) ?? Duration.zero) > before,
    );
    return (before: before, after: probe.latestPositionFor(focus)!);
  }

  Future<void> _swipe(double direction) async {
    final page = find.byType(PageView);
    final distance = tester.getSize(page).height;
    final gesture = await tester.startGesture(tester.getCenter(page));
    await gesture.moveBy(Offset(0, direction * distance * 0.7));
    await tester.pump(const Duration(milliseconds: 100));
    await gesture.up();
  }

  Future<void> _wait(
    bool Function() condition, {
    Duration timeout = const Duration(seconds: 30),
  }) async {
    final watch = Stopwatch()..start();
    while (!condition() && watch.elapsed < timeout) {
      await _tick();
    }
    if (!condition()) fail(_timeoutEvidence(timeout));
  }

  Future<void> _tick() async {
    await tester.pump(const Duration(milliseconds: 50));
    await Future<void>.delayed(const Duration(milliseconds: 20));
    _sampleCapacity();
  }

  void _sampleCapacity() {
    final mounted = find
        .byType(VideoPlayer, skipOffstage: false)
        .evaluate()
        .length;
    if (mounted > peakMountedPlayers) peakMountedPlayers = mounted;
    final capacity = videoPlaybackCapacityOf(graph.playback).inUse;
    if (capacity > peakControllerCapacity) peakControllerCapacity = capacity;
    unavailableWasVisible |= find
        .text('Video unavailable')
        .evaluate()
        .isNotEmpty;
  }

  String _timeoutEvidence(Duration timeout) =>
      'Validator rotation timed out after $timeout; '
      'state=${graph.cubit.state.runtimeType}, '
      'held=${fixture.hasHeldFirstRequest}, '
      'requests=${fixture.requests.length}, '
      'focus=${graph.focus.occurrences.map((item) => item.cause.name)}.';
}
