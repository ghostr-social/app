part of 'warp_bandwidth_recovery_scenario.dart';

Future<WarpSwipeBurst> _swipeIntoRecovery(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  _RecoveryFocus recovery,
) async {
  final count = recovery.frontier.readyDepth;
  expect(count, greaterThanOrEqualTo(2));
  final burst = await journey.swipeForward(
    tester,
    count: count,
    afterSequence: recovery.focus.sequence,
    cadence: deviceRapidSwipeCadence,
  );
  await journey.waitForFirstFrame(tester, burst.focuses.last);
  await journey.waitForPlaying(tester, burst.focuses.last);
  await journey.verifyReadyBurstPlayback(
    recovery.ready.snapshot,
    burst.focuses,
    burst.releases,
  );
  return burst;
}
