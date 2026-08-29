part of 'warp_feed_playback_journey.dart';

extension _WarpFeedPlaybackJourneyWaitCore on WarpFeedPlaybackJourney {
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
