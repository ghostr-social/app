part of 'warp_mixed_feed_readiness_scenario.dart';

Future<PlaybackFocus> _waitForFocus(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  int index, {
  int after = 0,
}) async {
  PlaybackFocus? result;
  await _waitUntil(tester, runtime, () {
    result = runtime.graph.focus.occurrenceAfter(
      runtime.events[index].id,
      after,
      cause: FeedFocusCause.userNavigation,
    );
    return result != null;
  });
  return result!;
}

Future<void> _waitForNativeFrame(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  PlaybackFocus focus,
) {
  return _waitUntil(
    tester,
    runtime,
    () => runtime.graph.telemetry.probe.firstFrameLatency(focus) != null,
  );
}

Future<void> _waitForFrameOrRescue(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  PlaybackFocus focus,
) {
  return _waitUntil(tester, runtime, () {
    return runtime.graph.telemetry.probe.firstFrameLatency(focus) != null ||
        _firstRescueAfter(runtime, focus.sequence) != null;
  });
}

Future<PlaybackFocus> _waitForHlsFocus(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  int after,
) async {
  PlaybackFocus? result;
  await _waitUntil(tester, runtime, () {
    result = runtime.graph.focus.occurrenceAfter(
      runtime.events[1].id,
      after,
      cause: FeedFocusCause.userNavigation,
    );
    return result != null || _firstRescueAfter(runtime, after) != null;
  });
  expect(_firstRescueAfter(runtime, after), isNull, reason: _evidence(runtime));
  return result!;
}

PlaybackFocus? _firstRescueAfter(WarpMixedFeedRuntime runtime, int sequence) {
  for (final candidate in runtime.graph.focus.occurrences) {
    if (candidate.sequence > sequence &&
        candidate.cause == FeedFocusCause.transportRescue) {
      return candidate;
    }
  }
  return null;
}

Future<void> _swipeUp(WidgetTester tester) async {
  final page = find.byType(PageView);
  final distance = tester.getSize(page).height;
  final gesture = await tester.startGesture(tester.getCenter(page));
  await gesture.moveBy(Offset(0, -distance * deviceRapidSwipeDistanceFraction));
  await tester.pump(deviceRapidSwipeGestureTarget);
  await gesture.up();
}

Future<void> _pumpFor(WidgetTester tester, Duration duration) async {
  final watch = Stopwatch()..start();
  while (watch.elapsed < duration) {
    await tester.pump(const Duration(milliseconds: 50));
    await Future<void>.delayed(const Duration(milliseconds: 20));
  }
}

Future<void> _waitUntil(
  WidgetTester tester,
  WarpMixedFeedRuntime runtime,
  bool Function() condition,
) async {
  final watch = Stopwatch()..start();
  while (!condition() && watch.elapsed < const Duration(seconds: 20)) {
    await tester.pump(const Duration(milliseconds: 50));
    await Future<void>.delayed(const Duration(milliseconds: 20));
  }
  if (!condition()) fail(_evidence(runtime));
}
