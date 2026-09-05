part of 'warp_ready_burst_scenario.dart';

Future<WarpFeedPlaybackJourney> _openFeed(WidgetTester tester) async {
  final journey = await WarpFeedPlaybackJourney.start(
    options: const WarpFeedDeviceOptions(
      events: SignedWarpFeedConfig(eventCount: 7),
      origin: WarpFeedOriginOptions(
        validator: ProgressiveOriginValidator.stableStrong,
        pacing: ProgressiveOriginPacing.perResponseDelay(
          Duration(milliseconds: 100),
        ),
      ),
    ),
  );
  addTearDown(journey.close);
  await tester.pumpWidget(journey.app);
  journey.load();
  await journey.waitForCaption(tester, 0);
  await journey.waitForPostCount(tester, 4);
  final startup = await journey.waitForPublishedFocus(tester, 0);
  await _expectMoving(tester, journey, startup);
  expect(
    journey.telemetry.probe.playingLatency(startup),
    lessThanOrEqualTo(deviceStartupTarget),
  );
  return journey;
}
