part of 'warp_ready_burst_scenario.dart';

Future<_ReadyBurstResult> _consumeReady(
  WidgetTester tester,
  _OpenedFeed opened,
  WarpReadyWindow ready,
) async {
  final journey = opened.journey;
  final startedAt = journey.telemetry.probe.elapsed;
  final swipe = await journey.swipeForward(
    tester,
    count: 3,
    afterSequence: journey.focusCursor,
    cadence: deviceRapidSwipeCadence,
  );
  journey.reportRapidCadence(startedAt, swipe);
  final focuses = swipe.focuses;
  final finalFocus = focuses.last;
  await journey.waitForFirstFrame(tester, finalFocus);
  await journey.waitForPlaying(tester, finalFocus);
  await journey.verifyReadyBurstPlayback(
    ready.snapshot,
    focuses,
    swipe.releases,
  );
  final replenished = await journey.waitForReplenishment(
    tester,
    finalFocus,
    afterRevision: ready.plan.revision,
  );
  return (focuses: focuses, finalFocus: finalFocus, replenished: replenished);
}

Future<PlaybackFocus> _consumeReplenished(
  WidgetTester tester,
  _OpenedFeed opened,
  WarpReadyWindow ready,
  _ReadyBurstResult burst,
) async {
  final journey = opened.journey;
  final releasedAt = await journey.swipeUp(tester);
  final next = await journey.waitForPublishedFocus(
    tester,
    burst.focuses.length + 1,
    afterSequence: burst.finalFocus.sequence,
  );
  await journey.waitForFirstFrame(tester, next);
  await journey.waitForPlaying(tester, next);
  await journey.telemetry.settled;
  expect(journey.isReadyIn(ready.snapshot, next), isFalse);
  await journey.verifyReadyPlayback(
    burst.replenished.snapshot,
    next,
    releasedAt,
  );
  final playback = journey.telemetry.probe;
  final position = playback.latestPositionFor(next)!;
  await journey.pumpFor(tester, const Duration(seconds: 1));
  journey.reportBurst(ready, burst.replenished, burst.focuses, next);
  await journey.reportSchedulingEvidence();
  expect(playback.latestPositionFor(next), greaterThan(position));
  expect(journey.hadPlaybackError, isFalse);
  expect(journey.focus.hadTransportRescue, isFalse);
  return next;
}
