part of 'warp_ready_burst_scenario.dart';

const _futurePaths = {'/next.mp4', '/third.mp4'};
const _usefulParallelPaths = {'/current.mp4', '/next.mp4', '/third.mp4'};

Future<_OpenedFeed> _openFeed(WidgetTester tester) async {
  final journey = await _startJourney();
  final liveness = journey.resources.origin.stageFirstChunks(_futurePaths);
  await _loadFeed(tester, journey);
  final startup = await _waitForStartup(tester, journey);
  final parallel = await _waitForParallel(tester, journey, liveness);
  final planning = await journey.waitForPublishedFocus(
    tester,
    0,
    afterSequence: startup.sequence,
    cause: FeedFocusCause.rosterChange,
  );
  return (
    journey: journey,
    startup: startup,
    planning: planning,
    parallel: parallel,
  );
}

Future<WarpFeedPlaybackJourney> _startJourney() async {
  final journey = await WarpFeedPlaybackJourney.start(
    eventCount: 7,
    validator: ProgressiveOriginValidator.stableStrong,
    dataUsage: DataUsageLevel.aggressive,
    pacing: const ProgressiveOriginPacing.perResponseDelay(
      Duration(milliseconds: 100),
    ),
  );
  addTearDown(journey.close);
  return journey;
}

Future<void> _loadFeed(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  await tester.pumpWidget(journey.app);
  journey.load();
  await journey.waitForCaption(tester, 0);
  await journey.waitForPostCount(tester, 4);
  expect(journey.futureRemotePaths(2).toSet(), _futurePaths);
}

Future<PlaybackFocus> _waitForStartup(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  final startup = await journey.waitForPublishedFocus(tester, 0);
  await journey.waitForFirstFrame(tester, startup);
  await journey.waitForPlaying(tester, startup);
  _expectStartup(journey, startup);
  return startup;
}

Future<ProgressiveRangedRequestPair> _waitForParallel(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  ProgressiveOriginFirstChunkRendezvous liveness,
) async {
  liveness.activate();
  await journey.waitForFirstChunkRendezvous(tester, liveness);
  return journey.waitForParallelBytes(tester, _usefulParallelPaths);
}

void _expectStartup(WarpFeedPlaybackJourney journey, PlaybackFocus focus) {
  expect(
    journey.telemetry.probe.playingLatency(focus),
    lessThanOrEqualTo(deviceStartupTarget),
  );
}

Future<WarpReadyWindow> _waitForReady(
  WidgetTester tester,
  _OpenedFeed opened,
) async {
  final journey = opened.journey;
  final generation = journey.focus.generationFor(opened.planning)!;
  final deliveryId = journey.telemetry.probe
      .sessionFor(opened.startup)!
      .deliveryId;
  final ready = await journey.waitForReadyBurstWindow(
    tester,
    generation,
    currentDeliveryId: deliveryId,
    minimumDepth: 3,
    afterSequence: opened.planning.sequence,
  );
  journey.reportPlan(ready.plan);
  journey.reportParallelPreparation(ready, opened.parallel);
  return ready;
}
