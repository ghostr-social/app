import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';

import 'device_playback_probe.dart';
import 'progressive_device_origin.dart';
import 'warp_feed_playback_journey.dart';
import 'warp_feed_player_stage_probe.dart';

part 'warp_progressive_loop_reopen_scenario_assertions.dart';

typedef _OpenedLoopFeed = ({
  WarpFeedPlaybackJourney journey,
  PlaybackFocus focus,
  PlaybackSession session,
  WarpFeedPlayerStageEvidence attempt,
});

Future<void> runWarpProgressiveLoopReopenScenario(WidgetTester tester) async {
  final opened = await _openLoopFeed(tester);
  final promotion = await opened.journey.waitForProgressivePromotion(
    tester,
    'current',
  );
  final observations = opened.journey.telemetry.probe.observations;
  final cursor = observations.isEmpty ? 0 : observations.last.sequence;
  final loop = await opened.journey.waitForProgressiveLoop(
    tester,
    opened.focus,
    afterSequence: cursor,
  );
  _expectPromotionAndLoop(promotion, loop);
  _expectOriginalPlaybackIdentity(opened);
  _expectHealthyPlayback(opened);
  _reportLoop(promotion, loop);
}

Future<_OpenedLoopFeed> _openLoopFeed(WidgetTester tester) async {
  final journey = await WarpFeedPlaybackJourney.start(
    eventCount: 3,
    validator: ProgressiveOriginValidator.stableStrong,
    dataUsage: DataUsageLevel.aggressive,
  );
  addTearDown(journey.close);
  await tester.pumpWidget(journey.app);
  journey.load();
  await journey.waitForCaption(tester, 0);
  await journey.waitForPostCount(tester, 3);
  final focus = await journey.waitForPublishedFocus(tester, 0);
  await journey.waitForFirstFrame(tester, focus);
  await journey.waitForPlaying(tester, focus);
  final session = journey.telemetry.probe.sessionFor(focus)!;
  final presented = journey.telemetry.probe.presentationFor(focus)!;
  final attempt = journey.playerStages.forPresentation(
    session.deliveryId,
    presented.elapsed,
  );
  expect(attempt, isNotNull);
  return (journey: journey, focus: focus, session: session, attempt: attempt!);
}
