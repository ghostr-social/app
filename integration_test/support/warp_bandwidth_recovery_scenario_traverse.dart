part of 'warp_bandwidth_recovery_scenario.dart';

typedef _WarmReturn = ({PlaybackFocus focus, int afterRevision});

Future<_RecoveryFocus> _traverseImpairedFeed(
  WidgetTester tester,
  _PacedFeed opened,
  _ImpairedFeed impaired,
) async {
  final returned = await _traverseImpairedWarmReturn(tester, opened, impaired);
  return _waitForRecoveryBaseline(tester, opened, impaired, returned);
}

Future<_WarmReturn> _traverseImpairedWarmReturn(
  WidgetTester tester,
  _PacedFeed opened,
  _ImpairedFeed impaired,
) async {
  final journey = opened.journey;
  final forward = await _swipeForwardUnderLoss(tester, journey, impaired);
  final reverse = await _swipeBackwardUnderLoss(tester, journey, forward);
  return _returnToThird(tester, opened, impaired, reverse.focuses.single);
}

Future<WarpSwipeBurst> _swipeForwardUnderLoss(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  _ImpairedFeed impaired,
) async {
  final forward = await journey.swipeForward(
    tester,
    count: 2,
    afterSequence: journey.focusCursor,
    cadence: deviceRapidSwipeCadence,
  );
  await journey.waitForFirstFrame(tester, forward.focuses.last);
  await journey.waitForPlaying(tester, forward.focuses.last);
  await journey.verifyReadyBurstPlayback(
    impaired.ready.snapshot,
    forward.focuses,
    forward.releases,
  );
  return forward;
}

Future<WarpSwipeBurst> _swipeBackwardUnderLoss(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  WarpSwipeBurst forward,
) async {
  final reverse = await journey.swipeBackward(
    tester,
    count: 1,
    afterSequence: forward.focuses.last.sequence,
    cadence: deviceRapidSwipeCadence,
  );
  await journey.waitForFirstFrame(tester, reverse.focuses.single);
  await journey.waitForPlaying(tester, reverse.focuses.single);
  journey.verifyReversePlayback(reverse.focuses, reverse.releases);
  return reverse;
}

Future<_WarmReturn> _returnToThird(
  WidgetTester tester,
  _PacedFeed opened,
  _ImpairedFeed impaired,
  PlaybackFocus from,
) async {
  final journey = opened.journey;
  final cursor =
      (await journey.evidence.page()).planPage.latestRetainedRevision;
  final releasedAt = await journey.swipeUp(tester);
  final focus = await journey.waitForPublishedFocus(
    tester,
    2,
    afterSequence: from.sequence,
  );
  await journey.waitForFirstFrame(tester, focus);
  await journey.waitForPlaying(tester, focus);
  await journey.verifyReadyPlayback(impaired.ready.snapshot, focus, releasedAt);
  expect(journey.focus.hadTransportRescue, isFalse);
  return (focus: focus, afterRevision: cursor);
}
