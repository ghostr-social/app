part of 'warp_bandwidth_recovery_scenario.dart';

const _transitionPaths = {
  '/current.mp4',
  '/next.mp4',
  '/third.mp4',
  '/fourth.mp4',
  '/fifth.mp4',
  '/sixth.mp4',
  '/seventh.mp4',
  '/eighth.mp4',
  '/ninth.mp4',
  '/tenth.mp4',
};

Future<_PacedFeed> _openPacedFeed(WidgetTester tester) async {
  final journey = await WarpFeedPlaybackJourney.start(
    eventCount: 10,
    validator: ProgressiveOriginValidator.stableStrong,
    dataUsage: DataUsageLevel.aggressive,
    pacing: const ProgressiveOriginPacing.sharedBandwidth(2500),
  );
  addTearDown(journey.close);
  final origin = journey.resources.origin;
  await _loadPacedFeed(tester, journey);
  final startup = await journey.waitForPublishedFocus(tester, 0);
  await journey.waitForFirstFrame(tester, startup);
  await journey.waitForPlaying(tester, startup);
  journey.reportStartup(startup);
  expect(
    journey.telemetry.probe.playingLatency(startup),
    lessThanOrEqualTo(deviceStartupTarget),
  );
  final profile = origin.currentLinkProfile!;
  await journey.waitForParallelRangedVideos(tester);
  final window = await journey.waitForConfirmedLinkWindow(
    tester,
    profile.generation,
    minimumDuration: const Duration(milliseconds: 750),
  );
  final decision = await journey.waitForDecision(
    tester,
    (item) =>
        item.observedAtMs > window.confirmedAtEpochMs &&
        item.networkThroughputBps > 0 &&
        item.appliesMeasuredNetworkRate,
  );
  debugPrint(
    'WARP_LINK baseline at=${decision.observedAtMs} '
    'throughput_bps=${decision.networkThroughputBps} '
    'planner_Bps=${decision.plannerNetworkRateBytesPerSecond}',
  );
  final generation = journey.focus.generationFor(startup)!;
  final plan = await journey.waitForPlan(
    tester,
    (item) =>
        item.observedAtMs == decision.observedAtMs &&
        item.networkStatusGeneration == 1 &&
        item.networkClass == WarpNetworkClass.wifi &&
        item.coversFocusGeneration(generation) &&
        item.plan.workBreadth >= 2,
  );
  final trigger = origin.armBandwidthChangeAfterNextConfirmedChunk(
    _transitionPaths,
    bandwidthKbps: 700,
  );
  journey.reportPlan(plan);
  return (
    journey: journey,
    startup: startup,
    focusGeneration: generation,
    lossTrigger: trigger,
    fastProfile: profile,
    baselineDecision: decision,
  );
}

Future<void> _loadPacedFeed(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  await tester.pumpWidget(journey.app);
  journey.load();
  await journey.waitForCaption(tester, 0);
  await journey.waitForPostCount(tester, 10);
}
