part of 'warp_feed_playback_journey.dart';

typedef WarpSwipeBurst = ({
  List<PlaybackFocus> focuses,
  List<Duration> releases,
});

extension WarpFeedPlaybackJourneyGestures on WarpFeedPlaybackJourney {
  Future<WarpSwipeBurst> swipeForward(
    WidgetTester tester, {
    required int count,
    required int afterSequence,
    Duration cadence = const Duration(milliseconds: 500),
  }) async {
    final expected = _forwardPostIds(count);
    final releases = <Duration>[];
    final watch = Stopwatch()..start();
    for (var index = 0; index < count; index += 1) {
      releases.add(await swipeUp(tester));
      await _paceSwipe(tester, cadence * (index + 1) - watch.elapsed);
    }
    await _wait(tester, () => _hasUserFocusOrder(expected, afterSequence));
    final focuses = _userFocusesAfter(afterSequence).take(count).toList();
    return (focuses: focuses, releases: releases);
  }

  List<String> _forwardPostIds(int count) {
    final state = cubit.state;
    if (state is! FeedLoaded ||
        state.posts.length - state.activeIndex <= count) {
      throw RangeError.value(count, 'count', 'exceeds forward history');
    }
    return state.posts
        .skip(state.activeIndex + 1)
        .take(count)
        .map((post) => post.id.value)
        .toList();
  }

  Future<WarpSwipeBurst> swipeBackward(
    WidgetTester tester, {
    required int count,
    required int afterSequence,
    Duration cadence = const Duration(milliseconds: 500),
  }) async {
    final state = cubit.state;
    if (state is! FeedLoaded || state.activeIndex < count) {
      throw RangeError.value(count, 'count', 'exceeds retained history');
    }
    final expected = List.generate(
      count,
      (offset) => state.posts[state.activeIndex - offset - 1].id.value,
    );
    final releases = <Duration>[];
    final watch = Stopwatch()..start();
    for (var index = 0; index < count; index += 1) {
      releases.add(await swipeDown(tester));
      await _paceSwipe(tester, cadence * (index + 1) - watch.elapsed);
    }
    await _wait(tester, () => _hasUserFocusOrder(expected, afterSequence));
    final focuses = _userFocusesAfter(afterSequence).take(count).toList();
    return (focuses: focuses, releases: releases);
  }

  Future<Duration> swipeUp(WidgetTester tester) => _swipe(tester, -1);

  Future<Duration> swipeDown(WidgetTester tester) => _swipe(tester, 1);

  Future<Duration> _swipe(WidgetTester tester, double direction) async {
    final page = find.byType(PageView);
    final distance = tester.getSize(page).height;
    final gesture = await tester.startGesture(tester.getCenter(page));
    await gesture.moveBy(
      Offset(0, direction * distance * deviceRapidSwipeDistanceFraction),
    );
    await tester.pump(deviceRapidSwipeGestureTarget);
    final releasedAt = telemetry.probe.elapsed;
    await gesture.up();
    return releasedAt;
  }

  Future<void> _paceSwipe(WidgetTester tester, Duration remaining) async {
    if (remaining > Duration.zero) await pumpFor(tester, remaining);
  }

  Future<void> pumpFor(WidgetTester tester, Duration duration) async {
    final watch = Stopwatch()..start();
    while (watch.elapsed < duration) {
      await _tickAndSample(tester);
    }
  }
}
