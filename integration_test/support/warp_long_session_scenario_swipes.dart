part of 'warp_long_session_scenario.dart';

enum _LongSwipeDirection { forward, backward }

extension _WarpLongSessionSwipes on _WarpLongSessionDriver {
  Future<void> _swipeLeg(_LongSwipeDirection direction, int count) async {
    final expected = _expectedFocuses(direction, count);
    PlaybackFocus? latest;
    var cursor = 0;
    while (cursor < expected.length) {
      final after = graph.focus.occurrences.last.sequence;
      await _gesture(direction);
      final intended = await _waitForFocus(expected[cursor], after);
      latest = await _waitForDecodedOrRescue(intended);
      cursor = _recordSettledFocus(expected, cursor, intended, latest);
    }
    await _expectPlaybackAdvances(latest!);
  }

  List<String> _expectedFocuses(_LongSwipeDirection direction, int count) {
    final state = graph.cubit.state as FeedLoaded;
    if (direction == _LongSwipeDirection.forward) {
      if (state.posts.length - state.activeIndex <= count) {
        throw RangeError.value(count, 'count', 'exceeds forward roster');
      }
      return state.posts
          .skip(state.activeIndex + 1)
          .take(count)
          .map((post) => post.id.value)
          .toList();
    }
    if (state.activeIndex < count) {
      throw RangeError.value(count, 'count', 'exceeds backward roster');
    }
    return List.generate(
      count,
      (offset) => state.posts[state.activeIndex - offset - 1].id.value,
    );
  }

  Future<PlaybackFocus> _waitForFocus(String expected, int after) async {
    PlaybackFocus? focus;
    await _wait(() {
      focus = graph.focus.occurrenceAfter(
        expected,
        after,
        cause: FeedFocusCause.userNavigation,
      );
      return focus != null;
    }, awaiting: 'userFocus=$expected after=$after');
    return focus!;
  }

  List<PlaybackFocus> _userFocusesAfter(int sequence) => graph.focus.occurrences
      .where(
        (focus) =>
            focus.sequence > sequence &&
            focus.cause == FeedFocusCause.userNavigation,
      )
      .toList();

  Future<void> _gesture(_LongSwipeDirection direction) async {
    final page = find.byType(PageView);
    final sign = direction == _LongSwipeDirection.forward ? -1 : 1;
    final gesture = await tester.startGesture(tester.getCenter(page));
    await gesture.moveBy(Offset(0, sign * tester.getSize(page).height * 0.23));
    await tester.pump(deviceRapidSwipeGestureTarget);
    _sampleVisibleState();
    await gesture.up();
    await _pumpFor(deviceRapidSwipeCadence);
  }

  Future<void> _expectPlaybackAdvances(PlaybackFocus focus) async {
    final before = graph.telemetry.probe.latestPositionFor(focus)!;
    await _pumpFor(const Duration(seconds: 1));
    expect(
      graph.telemetry.probe.latestPositionFor(focus),
      isNot(equals(before)),
    );
  }
}
