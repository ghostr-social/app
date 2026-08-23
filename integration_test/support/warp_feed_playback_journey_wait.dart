part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyWait on WarpFeedPlaybackJourney {
  Future<PlaybackFocus> waitForPublishedFocus(
    WidgetTester tester,
    int index,
  ) async {
    final eventId = events[index].id;
    await _wait(tester, () => focus.publishedFor(eventId) != null);
    return focus.publishedFor(eventId)!;
  }

  Future<void> waitForCaption(WidgetTester tester, int index) {
    return _wait(tester, () {
      return find.text('WARP signed ${_labels[index]}').evaluate().isNotEmpty;
    });
  }

  Future<void> waitForPlaying(WidgetTester tester, PlaybackFocus focus) {
    return _wait(tester, () => telemetry.probe.playingLatency(focus) != null);
  }

  Future<void> waitForFirstFrame(WidgetTester tester, PlaybackFocus focus) {
    return _wait(
      tester,
      () => telemetry.probe.firstFrameLatency(focus) != null,
    );
  }

  Future<void> waitForPreparation(WidgetTester tester) {
    return _wait(tester, () => preparation.maximumStructuralDepth >= 1);
  }

  Future<void> waitForParallelRangedVideos(WidgetTester tester) {
    return _wait(tester, () => resources.origin.hadParallelRangedVideos);
  }

  Future<void> swipeUp(WidgetTester tester) => _swipe(tester, -600);

  Future<void> swipeDown(WidgetTester tester) => _swipe(tester, 600);

  Future<void> _swipe(WidgetTester tester, double dy) async {
    await tester.drag(find.byType(PageView), Offset(0, dy));
    await pumpFor(tester, const Duration(milliseconds: 200));
  }

  Future<void> pumpFor(WidgetTester tester, Duration duration) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < duration) {
      await _tick(tester);
    }
  }

  Future<void> _wait(
    WidgetTester tester,
    bool Function() condition, {
    Duration timeout = const Duration(seconds: 15),
  }) async {
    final watch = Stopwatch()..start();
    while (!condition() && watch.elapsed < timeout) {
      await _tick(tester);
    }
    if (!condition()) fail(_timeoutEvidence(timeout));
  }

  String _timeoutEvidence(Duration timeout) {
    return 'WARP feed condition timed out after $timeout; '
        'state=${cubit.state.runtimeType}, '
        'relayConnections=${relay.acceptedConnections}, '
        'relayRequests=${relay.requestMessages}, '
        'videoSubscriptions=${relay.videoSubscriptions}, '
        'eventsSent=${relay.eventsSent}, '
        'rust=${graph.rustProbe.evidence}, '
        'originRequests=${resources.origin.requests.length}, '
        'structuralDepth=${preparation.maximumStructuralDepth}, '
        'filters=${relay.requestedFilters}.';
  }
}

Future<void> _tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 50));
  await Future<void>.delayed(const Duration(milliseconds: 20));
}

const _labels = ['current', 'next', 'third'];
