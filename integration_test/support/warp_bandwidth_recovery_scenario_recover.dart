part of 'warp_bandwidth_recovery_scenario.dart';

Future<void> _recoverSharedLink(
  WidgetTester tester,
  _PacedFeed opened,
  _ImpairedFeed impaired,
  _RecoveryFocus recovery,
) async {
  final journey = opened.journey;
  final recovered = await _measureRecovery(tester, opened, impaired, recovery);
  await _verifyRecoveredPlayback(tester, journey, recovered);
}

Future<void> _verifyRecoveredPlayback(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  _RecoveredEvidence recovered,
) async {
  final focus = recovered.focus;
  final nextIndex = _nextFixtureIndex(journey, focus);
  expect(
    journey.futureRemotePaths(1).single,
    recovered.frontierPath,
    reason: 'recovery must replenish the captured non-ready frontier',
  );
  final replenished = await journey.waitForReplenishment(
    tester,
    focus,
    afterRevision: recovered.plan.revision,
  );
  final releasedAt = await journey.swipeUp(tester);
  final next = await journey.waitForPublishedFocus(
    tester,
    nextIndex,
    afterSequence: focus.sequence,
  );
  await journey.waitForFirstFrame(tester, next);
  await journey.waitForPlaying(tester, next);
  await journey.verifyReadyPlayback(replenished.snapshot, next, releasedAt);
  await _verifyPlaybackAdvances(tester, journey, next);
  await _verifyBandwidthAcceptance(tester, journey);
}

int _nextFixtureIndex(WarpFeedPlaybackJourney journey, PlaybackFocus focus) {
  final current = journey.events.indexWhere(
    (event) => event.id == focus.videoId.value,
  );
  final next = current + 1;
  if (current < 0 || next >= journey.events.length) {
    throw StateError('Recovered focus has no next fixture item.');
  }
  return next;
}

Future<void> _verifyPlaybackAdvances(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  PlaybackFocus focus,
) async {
  final position = journey.telemetry.probe.latestPositionFor(focus)!;
  await journey.pumpFor(tester, const Duration(seconds: 1));
  expect(
    journey.telemetry.probe.latestPositionFor(focus),
    greaterThan(position),
  );
}
