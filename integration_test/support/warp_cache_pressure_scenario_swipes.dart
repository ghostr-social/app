part of 'warp_cache_pressure_scenario.dart';

extension _WarpCachePressureSwipes on _WarpCachePressureDriver {
  Future<PlaybackFocus> _swipeForward() => _swipe(1);

  Future<PlaybackFocus> _returnToCold() async {
    PlaybackFocus? focus;
    for (var count = 0; count < forwardHandoffs; count += 1) {
      focus = await _swipe(-1);
      _expectWithinBudget(await _cacheCoverage());
    }
    expect(focus?.videoId.value, session.events.first.id);
    return focus!;
  }

  Future<PlaybackFocus> _swipe(int direction) async {
    final state = graph.cubit.state as FeedLoaded;
    final target = state.activeIndex + direction;
    final eventId = state.posts[target].id.value;
    final after = graph.focus.occurrences.last.sequence;
    await _gesture(direction);
    final focus = await _waitForFocus(
      eventId,
      after,
      cause: FeedFocusCause.userNavigation,
    );
    await _waitForDecodedPlayback(focus);
    return focus;
  }

  Future<void> _gesture(int direction) async {
    final page = find.byType(PageView);
    final gesture = await tester.startGesture(tester.getCenter(page));
    final distance = tester.getSize(page).height * 0.23 * -direction;
    await gesture.moveBy(Offset(0, distance));
    await tester.pump(const Duration(milliseconds: 16));
    _sampleVisibleState();
    await gesture.up();
    await _pumpFor(const Duration(milliseconds: 150));
  }

  Future<PlaybackFocus> _waitForFocus(
    String eventId,
    int after, {
    FeedFocusCause? cause,
  }) async {
    PlaybackFocus? found;
    await _wait(() {
      found = graph.focus.occurrenceAfter(eventId, after, cause: cause);
      return found != null;
    });
    return found!;
  }
}
