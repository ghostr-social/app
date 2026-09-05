import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:video_player/video_player.dart';

import 'device_playback_probe.dart';
import 'device_qoe_targets.dart';
import 'progressive_device_origin.dart';
import 'warp_feed_playback_journey.dart';

part 'warp_ready_burst_scenario_open.dart';
part 'warp_ready_burst_scenario_retention.dart';

Future<void> runWarpReadyBurstScenario(WidgetTester tester) async {
  final journey = await _openFeed(tester);
  await journey.waitForPreparation(tester);
  final startedAt = journey.telemetry.probe.elapsed;
  final burst = await journey.swipeForward(
    tester,
    count: 3,
    afterSequence: journey.focusCursor,
    cadence: deviceRapidSwipeCadence,
  );
  journey.reportRapidCadence(startedAt, burst);
  await _expectMoving(tester, journey, burst.focuses.last);
  await journey.waitForPreparation(tester);
  _expectPlayerBound(tester);
  await journey.reportSchedulingEvidence();
}

Future<void> _expectMoving(
  WidgetTester tester,
  WarpFeedPlaybackJourney journey,
  PlaybackFocus focus,
) async {
  await journey.waitForFirstFrame(tester, focus);
  await journey.waitForPlaying(tester, focus);
  final before = journey.telemetry.probe.latestPositionFor(focus)!;
  await journey.pumpFor(tester, const Duration(milliseconds: 500));
  expect(journey.telemetry.probe.latestPositionFor(focus), greaterThan(before));
  expect(journey.hadPlaybackError, isFalse);
  expect(journey.focus.hadTransportRescue, isFalse);
  expect(find.text('Video unavailable'), findsNothing);
  _expectPlayerBound(tester);
}

void _expectPlayerBound(WidgetTester tester) {
  final players = find.byType(VideoPlayer, skipOffstage: false);
  expect(players.evaluate().length, inInclusiveRange(1, 2));
}
