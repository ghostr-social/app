part of 'warp_feed_playback_journey.dart';

extension WarpFeedPlaybackJourneyWait on WarpFeedPlaybackJourney {
  Future<PlaybackFocus> waitForPublishedFocus(
    WidgetTester tester,
    int index, {
    int afterSequence = 0,
    FeedFocusCause cause = FeedFocusCause.userNavigation,
  }) async {
    final eventId = events[index].id;
    PlaybackFocus? occurrence;
    await _wait(tester, () {
      occurrence = focus.occurrenceAfter(eventId, afterSequence, cause: cause);
      return occurrence != null;
    });
    return occurrence!;
  }

  int get focusCursor {
    final occurrences = focus.occurrences;
    return occurrences.isEmpty ? 0 : occurrences.last.sequence;
  }

  Future<List<PlaybackFocus>> waitForUserFocusOrder(
    WidgetTester tester,
    List<int> indices, {
    required int afterSequence,
  }) async {
    final expected = indices.map((index) => events[index].id).toList();
    await _wait(tester, () => _hasUserFocusOrder(expected, afterSequence));
    return _userFocusesAfter(afterSequence).take(expected.length).toList();
  }

  Future<void> waitForCaption(WidgetTester tester, int index) {
    return _wait(tester, () {
      return find.text('WARP signed ${_labels[index]}').evaluate().isNotEmpty;
    });
  }

  Future<void> waitForPostCount(WidgetTester tester, int count) {
    return _wait(tester, () {
      final state = cubit.state;
      return state is FeedLoaded && state.posts.length >= count;
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

  Future<void> waitForPreparation(
    WidgetTester tester, {
    int minimumReadyDepth = 1,
  }) async {
    try {
      await _wait(
        tester,
        () =>
            preparation.observations.isNotEmpty &&
            preparation.latest.readyDepth >= minimumReadyDepth,
      );
    } on Object {
      await reportSchedulingEvidence();
      rethrow;
    }
  }

  Future<void> waitForParallelRangedVideos(WidgetTester tester) {
    return _wait(tester, () => resources.origin.hadParallelRangedVideos);
  }

  Future<ProgressiveRangedRequestPair> waitForParallelBytes(
    WidgetTester tester,
    Iterable<String> paths,
  ) async {
    await _wait(
      tester,
      () => resources.origin.rangedByteOverlap(paths) != null,
    );
    return resources.origin.rangedByteOverlap(paths)!;
  }

  Future<void> swipeUp(WidgetTester tester) => _swipe(tester, -1);

  Future<void> swipeDown(WidgetTester tester) => _swipe(tester, 1);

  Future<void> _swipe(WidgetTester tester, double direction) async {
    final page = find.byType(PageView);
    final distance = tester.getSize(page).height * 0.65;
    await tester.fling(page, Offset(0, direction * distance), 1800);
    await pumpFor(tester, const Duration(milliseconds: 200));
  }

  Future<void> pumpFor(WidgetTester tester, Duration duration) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < duration) {
      await _tickAndSample(tester);
    }
  }

  Future<void> _wait(
    WidgetTester tester,
    bool Function() condition, {
    Duration timeout = const Duration(seconds: 15),
  }) async {
    final watch = Stopwatch()..start();
    while (!condition() && watch.elapsed < timeout) {
      await _tickAndSample(tester);
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
        'origin=${_originEvidence()}, '
        'structuralDepth=${preparation.maximumStructuralDepth}, '
        'readyDepth=${preparation.maximumReadyDepth}, '
        'focuses=${_focusEvidence()}, '
        'delivery=${graph.deliveryProbe.evidence}, '
        'playbackErrors=${playbackErrorSamples.length}, '
        'filters=${relay.requestedFilters}.';
  }

  String _focusEvidence() {
    return focus.occurrences
        .map((item) {
          final generation = focus.generationFor(item);
          final reason = item.rescue?.reason.name ?? 'none';
          return '${item.videoId.value}:${item.cause.name}:$reason:'
              '${item.sequence}:$generation';
        })
        .join('|');
  }

  List<PlaybackFocus> _userFocusesAfter(int sequence) {
    return focus.occurrences
        .where(
          (item) =>
              item.sequence > sequence &&
              item.cause == FeedFocusCause.userNavigation,
        )
        .toList();
  }

  bool _hasUserFocusOrder(List<String> expected, int sequence) {
    final actual = _userFocusesAfter(sequence);
    if (actual.length < expected.length) return false;
    return Iterable<int>.generate(
      expected.length,
    ).every((index) => actual[index].videoId.value == expected[index]);
  }

  Future<void> _tickAndSample(WidgetTester tester) async {
    await _tick(tester);
    if (find.text('Video unavailable').evaluate().isEmpty) return;
    if (playbackErrorSamples.length == 256) playbackErrorSamples.removeAt(0);
    playbackErrorSamples.add(telemetry.probe.elapsed);
  }
}

Future<void> _tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 50));
  await Future<void>.delayed(const Duration(milliseconds: 20));
}

const _labels = [
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
