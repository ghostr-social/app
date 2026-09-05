import 'package:flutter/foundation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';

import 'device_playback_probe.dart';
import 'progressive_device_origin.dart';
import 'warp_feed_playback_journey.dart';
import 'warp_feed_player_stage_probe.dart';

part 'warp_malformed_range_scenario_assertions.dart';
part 'warp_malformed_range_scenario_wait.dart';

Future<void> runWarpMalformedRangeScenario(WidgetTester tester) async {
  final scenario = await _MalformedRangeScenario.open();
  addTearDown(scenario.close);
  // Keep a later valid whole-file fallback from supplying this rejection case.
  final heldNext = scenario.journey.resources.origin.holdBeforeFirstBody({
    '/next.mp4',
  }, timeout: const Duration(minutes: 2));
  addTearDown(heldNext.release);
  await scenario.mount(tester);
  await scenario.waitForRejectedRange(tester);
  scenario.expectRejectedRangeIsNotReady();
  final next = await scenario.navigatePastRejectedVideo(tester);
  await scenario.expectDecodedNext(tester, next);
  scenario.expectRejectedRangeIsNotReady();
  await scenario.journey.waitForOriginQuiescence(tester, ['next']);
  scenario.expectBoundedOriginWork();
}

final class _MalformedRangeScenario {
  _MalformedRangeScenario._(this.journey);

  static Future<_MalformedRangeScenario> open() async {
    final journey = await WarpFeedPlaybackJourney.start(
      options: const WarpFeedDeviceOptions(
        events: SignedWarpFeedConfig(eventCount: 4),
        dataUsage: DataUsageLevel.aggressive,
        origin: WarpFeedOriginOptions(
          validator: ProgressiveOriginValidator.stableStrong,
          rangeSemanticsById: {
            'next': ProgressiveOriginRangeSemantics.malformed,
          },
        ),
      ),
    );
    return _MalformedRangeScenario._(journey);
  }

  final WarpFeedPlaybackJourney journey;

  FeedLoaded get feed => journey.cubit.state as FeedLoaded;
  PlaybackDeliveryId get malformedId => _deliveryIdAt(1);

  PlaybackDeliveryId _deliveryIdAt(int index) {
    return feed.posts[index].media.playbackDeliveryId!;
  }

  Future<void> mount(WidgetTester tester) async {
    await tester.pumpWidget(journey.app);
    journey.load();
    await journey.waitForCaption(tester, 0);
    await journey.waitForPostCount(tester, 4);
    final focus = await journey.waitForPublishedFocus(tester, 0);
    await journey.waitForFirstFrame(tester, focus);
    await journey.waitForPlaying(tester, focus);
  }

  Future<void> close() => journey.close();
}
