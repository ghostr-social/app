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
}
