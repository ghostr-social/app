import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:video_player/video_player.dart';

import 'device_playback_probe.dart';
import 'warp_feed_player_stage_probe.dart';
import 'progressive_device_origin.dart';
import 'warp_feed_playback_journey.dart';

Future<void> runWarpPlayerVerifiedRescueScenario(WidgetTester tester) async {
  final journey = await WarpFeedPlaybackJourney.start(
    options: const WarpFeedDeviceOptions(
      events: SignedWarpFeedConfig(eventCount: 4),
      dataUsage: DataUsageLevel.aggressive,
      origin: WarpFeedOriginOptions(
        validator: ProgressiveOriginValidator.stableStrong,
      ),
    ),
  );
  final gate = journey.resources.origin.holdBeforeFirstBody({
    '/next.mp4',
  }, timeout: const Duration(seconds: 30));
  addTearDown(() async {
    gate.release();
    await journey.close();
  });
  await _mount(tester, journey);
  await _waitForGate(tester, journey, gate);
  await _recoverIntendedVideo(tester, journey, gate);
}

Future<void> _mount(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  await tester.pumpWidget(journey.app);
  journey.load();
  await journey.waitForCaption(tester, 0);
  await journey.waitForPostCount(tester, 4);
  final current = await journey.waitForPublishedFocus(tester, 0);
  await journey.waitForFirstFrame(tester, current);
  await journey.waitForPlaying(tester, current);
  final distant = journey.focus.deliveryForEvent(journey.events[3].id)!;
  expect(journey.playerStages.attemptsFor(distant), isEmpty);
}

Future<void> _waitForGate(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  ProgressiveOriginPreBodyGate gate,
) async {
  final watch = Stopwatch()..start();
  while (!gate.isReached && watch.elapsed < const Duration(seconds: 5)) {
    await journey.pumpFor(tester, const Duration(milliseconds: 50));
  }
  expect(gate.isReached, isTrue);
  expect(gate.timedOut, isFalse);
}

Future<void> _recoverIntendedVideo(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  ProgressiveOriginPreBodyGate gate,
) async {
  final cursor = journey.focusCursor;
  await journey.swipeUp(tester);
  final intended = await journey.waitForPublishedFocus(
    tester,
    1,
    afterSequence: cursor,
  );
  await journey.pumpFor(tester, const Duration(milliseconds: 600));
  await journey.waitForCaption(tester, 1);
  expect(journey.focus.hadTransportRescue, isFalse);
  expect(gate.timedOut, isFalse);
  gate.release();
  await journey.waitForFirstFrame(tester, intended);
  await journey.waitForPlaying(tester, intended);
  await _verifyRecovery(tester, journey);
}

Future<void> _verifyRecovery(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
) async {
  final intended = await journey.waitForPublishedFocus(tester, 1);
  final position = journey.telemetry.probe.latestPositionFor(intended)!;
  await journey.pumpFor(tester, const Duration(milliseconds: 500));
  expect(
    journey.telemetry.probe.latestPositionFor(intended),
    greaterThan(position),
  );
  expect(journey.focus.hadTransportRescue, isFalse);
  expect(find.text('Video unavailable'), findsNothing);
  expect(
    find.byType(VideoPlayer, skipOffstage: false).evaluate().length,
    inInclusiveRange(1, 2),
  );
  await journey.reportSchedulingEvidence();
}
