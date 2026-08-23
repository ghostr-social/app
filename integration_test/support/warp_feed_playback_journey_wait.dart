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
    return _wait(tester, () => preparation.maximumReadyDepth >= 1);
  }

  Future<void> waitForParallelRangedVideos(WidgetTester tester) {
    return _wait(tester, () => resources.origin.hadParallelRangedVideos);
  }

  Future<void> swipeUp(WidgetTester tester) => _swipe(tester, -1);

  Future<void> _swipe(WidgetTester tester, double direction) async {
    final page = find.byType(PageView);
    final distance = tester.getSize(page).height * 0.23;
    await tester.drag(page, Offset(0, direction * distance));
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
        'active=${cubit.state is FeedLoaded ? (cubit.state as FeedLoaded).activeIndex : 'na'}, '
        'relayConnections=${relay.acceptedConnections}, '
        'relayRequests=${relay.requestMessages}, '
        'videoSubscriptions=${relay.videoSubscriptions}, '
        'eventsSent=${relay.eventsSent}, '
        'rust=${graph.rustProbe.evidence}, '
        'originRequests=${resources.origin.requests.length}, '
        'structuralDepth=${preparation.maximumStructuralDepth}, '
        'readyDepth=${preparation.maximumReadyDepth}, '
        'filters=${relay.requestedFilters}.';
  }
}

Future<void> _tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 50));
  await Future<void>.delayed(const Duration(milliseconds: 20));
}

const _labels = ['current', 'next', 'third'];
