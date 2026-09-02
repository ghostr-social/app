import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import 'device_playback_probe.dart';
import 'progressive_device_origin.dart';
import 'warp_feed_playback_journey.dart';
import 'warp_feed_player_stage_probe.dart';
import 'warp_feed_preparation_probe.dart';

part 'warp_origin_timeout_fallback_scenario_assertions.dart';
part 'warp_origin_timeout_fallback_scenario_evidence.dart';
part 'warp_origin_timeout_fallback_scenario_quiescence.dart';
part 'warp_origin_timeout_fallback_scenario_release.dart';
part 'warp_origin_timeout_fallback_scenario_wait.dart';

Future<void> runWarpOriginTimeoutFallbackScenario(WidgetTester tester) async {
  final scenario = await _OriginTimeoutFallbackScenario.open();
  addTearDown(scenario.close);
  await scenario.mount(tester);
  final evidence = await scenario.waitForVerifiedFallback(tester);
  scenario.expectBoundedFailover(evidence);
  await scenario.expectTransientPrimaryFailure(tester);
  await scenario.releaseLatePrimary(tester, evidence);
  final focus = await scenario.swipeToIntendedNext(tester);
  await scenario.expectDecodedNext(tester, focus, evidence);
  await scenario.expectQuiescentAfterUnmount(tester);
}

final class _OriginTimeoutFallbackScenario {
  _OriginTimeoutFallbackScenario._(
    this.journey,
    this.primaryGate,
    this.decisionBaseline,
  );

  static Future<_OriginTimeoutFallbackScenario> open() async {
    final journey = await WarpFeedPlaybackJourney.start(
      options: const WarpFeedDeviceOptions(
        events: SignedWarpFeedConfig(
          eventCount: 3,
          candidateLayout: WarpFeedCandidateLayout.nextWithRescue,
        ),
        dataUsage: DataUsageLevel.aggressive,
        origin: WarpFeedOriginOptions(
          validator: ProgressiveOriginValidator.stableStrong,
        ),
      ),
    );
    final history = await journey.evidence.decisions();
    final gate = journey.resources.origin.holdBeforeFirstBody({
      '/next.mp4',
    }, timeout: const Duration(seconds: 45));
    final baseline = history.records.isEmpty
        ? 0
        : history.records.last.sequence;
    return _OriginTimeoutFallbackScenario._(journey, gate, baseline);
  }

  final WarpFeedPlaybackJourney journey;
  final ProgressiveOriginPreBodyGate primaryGate;
  final int decisionBaseline;

  FeedLoaded get feed => journey.cubit.state as FeedLoaded;
  PlaybackDeliveryId get nextId => feed.posts[1].media.playbackDeliveryId!;

  Future<void> mount(WidgetTester tester) async {
    await tester.pumpWidget(journey.app);
    journey.load();
    await journey.waitForCaption(tester, 0);
    await journey.waitForPostCount(tester, 3);
    final focus = await journey.waitForPublishedFocus(tester, 0);
    await journey.waitForFirstFrame(tester, focus);
    await journey.waitForPlaying(tester, focus);
  }

  Future<void> close() async {
    primaryGate.release();
    await journey.close();
  }
}
