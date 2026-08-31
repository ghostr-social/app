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
    minimumDuration: const Duration(milliseconds: 1500),
  );
  final generation = journey.focus.generationFor(startup)!;
  final paired = await _waitForBaselinePair(
    tester,
    journey,
    window.confirmedAtEpochMs,
    generation,
  );
  _reportBaseline(paired.decision);
  final trigger = origin.armBandwidthChangeAfterNextConfirmedChunk(
    _transitionPaths,
    bandwidthKbps: 700,
  );
  journey.reportPlan(paired.plan);
  return (
    journey: journey,
    startup: startup,
    focusGeneration: generation,
    lossTrigger: trigger,
    fastProfile: profile,
    baselineDecision: paired.decision,
    baselinePlanRevision: paired.plan.revision,
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
