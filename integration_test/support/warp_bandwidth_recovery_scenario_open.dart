part of 'warp_bandwidth_recovery_scenario.dart';

Future<WarpFeedPlaybackJourney> _openPacedFeed(WidgetTester tester) async {
  final journey = await WarpFeedPlaybackJourney.start(
    options: const WarpFeedDeviceOptions(
      events: SignedWarpFeedConfig(eventCount: 10),
      dataUsage: DataUsageLevel.aggressive,
      origin: WarpFeedOriginOptions(
        validator: ProgressiveOriginValidator.stableStrong,
        pacing: ProgressiveOriginPacing.sharedBandwidth(2500),
      ),
    ),
  );
  addTearDown(journey.close);
  await tester.pumpWidget(journey.app);
  journey.load();
  await journey.waitForCaption(tester, 0);
  await journey.waitForPostCount(tester, 10);
  await _expectMoving(
    tester,
    journey,
    await journey.waitForPublishedFocus(tester, 0),
  );
  return journey;
}

Future<WarpDecisionRecord> _baseline(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) => journey.waitForDecision(
  tester,
  (decision) =>
      decision.networkThroughputBps >= 1000000 &&
      decision.appliesMeasuredNetworkRate,
);

Future<void> _expectMoving(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  PlaybackFocus focus,
) async {
  await journey.waitForFirstFrame(tester, focus);
  await journey.waitForPlaying(tester, focus);
  final position = journey.telemetry.probe.latestPositionFor(focus)!;
  await journey.pumpFor(tester, const Duration(milliseconds: 500));
  expect(
    journey.telemetry.probe.latestPositionFor(focus),
    greaterThan(position),
  );
  expect(find.text('Video unavailable'), findsNothing);
  expect(journey.focus.hadTransportRescue, isFalse);
}
