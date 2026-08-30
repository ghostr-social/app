part of 'warp_ready_burst_scenario.dart';

Future<void> runWarpAdaptiveWarmBackScenario(WidgetTester tester) async {
  final opened = await _openFeed(tester);
  final ready = await _waitForReady(tester, opened);
  final burst = await _consumeReady(tester, opened, ready);
  final retained = _pageControllers(tester, burst.focuses);
  final next = await _consumeReplenished(tester, opened, ready, burst);
  _expectMountedControllers(tester, retained);
  final reverse = await _reversePlayback(tester, opened.journey, next);
  await _verifyWarmReuse(tester, opened.journey, retained, reverse.focuses);
}

Map<PlaybackVideoId, VideoPlayerController> _pageControllers(
  WidgetTester tester,
  Iterable<PlaybackFocus> focuses,
) {
  return {
    for (final focus in focuses)
      focus.videoId: _pageController(tester, focus.videoId),
  };
}

VideoPlayerController _pageController(
  WidgetTester tester,
  PlaybackVideoId videoId,
) {
  final page = find.byKey(ValueKey(videoId.value), skipOffstage: false);
  final player = find.descendant(
    of: page,
    matching: find.byType(VideoPlayer, skipOffstage: false),
    skipOffstage: false,
  );
  expect(player, findsOneWidget, reason: videoId.value);
  return tester.widget<VideoPlayer>(player).controller;
}

void _expectMountedControllers(
  WidgetTester tester,
  Map<PlaybackVideoId, VideoPlayerController> retained,
) {
  final mounted = tester
      .widgetList<VideoPlayer>(find.byType(VideoPlayer, skipOffstage: false))
      .map((player) => player.controller);
  for (final entry in retained.entries) {
    expect(
      mounted.any((controller) => identical(controller, entry.value)),
      isTrue,
      reason: entry.key.value,
    );
  }
}

Future<void> _verifyWarmReuse(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  Map<PlaybackVideoId, VideoPlayerController> retained,
  List<PlaybackFocus> reverse,
) async {
  final finalFocus = reverse.last;
  final controller = _pageController(tester, finalFocus.videoId);
  expect(
    identical(controller, retained[finalFocus.videoId]),
    isTrue,
    reason: finalFocus.videoId.value,
  );
  expect(controller.value.isInitialized, isTrue);
  final position = journey.telemetry.probe.latestPositionFor(finalFocus)!;
  await journey.pumpFor(tester, const Duration(seconds: 1));
  expect(
    journey.telemetry.probe.latestPositionFor(finalFocus),
    greaterThan(position),
  );
  expect(journey.hadPlaybackError, isFalse);
  expect(journey.focus.hadTransportRescue, isFalse);
  expect(find.text('Video unavailable'), findsNothing);
}
