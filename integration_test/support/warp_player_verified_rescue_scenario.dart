import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';

import 'device_playback_probe.dart';
import 'inactive_prepared_playback_gate.dart';
import 'progressive_device_origin.dart';
import 'warp_feed_playback_journey.dart';
import 'warp_feed_player_stage_probe.dart';
import 'warp_feed_preparation_probe.dart';
import 'warp_feed_surface.dart';

part 'warp_player_verified_rescue_scenario_evidence.dart';
part 'warp_player_verified_rescue_scenario_predicate.dart';
part 'warp_player_verified_rescue_scenario_wait.dart';

Future<void> runWarpPlayerVerifiedRescueScenario(WidgetTester tester) async {
  final scenario = await _PlayerVerifiedRescueScenario.open();
  addTearDown(scenario.close);
  await scenario.mount(tester);
  scenario.verifyCandidateEvidence();
  final rescue = await scenario.swipeToRescue(tester);
  await scenario.verifyVisibleFrame(tester, rescue);
}

final class _PlayerVerifiedRescueScenario {
  _PlayerVerifiedRescueScenario._(this.journey, this.bodyGate, this.playerGate);

  static Future<_PlayerVerifiedRescueScenario> open() async {
    final journey = await WarpFeedPlaybackJourney.start(
      eventCount: 4,
      validator: ProgressiveOriginValidator.stableStrong,
      dataUsage: DataUsageLevel.aggressive,
    );
    final bodyGate = journey.resources.origin.holdBeforeFirstBody({
      '/next.mp4',
    }, timeout: const Duration(seconds: 30));
    final blockedId = PlaybackVideoId.parse(journey.events[2].id);
    final playerGate = InactivePreparedPlaybackGate(
      journey.playback,
      blockedId,
    );
    return _PlayerVerifiedRescueScenario._(journey, bodyGate, playerGate);
  }

  final WarpFeedPlaybackJourney journey;
  final ProgressiveOriginPreBodyGate bodyGate;
  final InactivePreparedPlaybackGate playerGate;
  late WarpFeedPlayerStageEvidence readyStage;

  PlaybackAssetAuthority get readyAuthority => readyStage.authority;

  Future<void> mount(WidgetTester tester) async {
    final surface = WarpFeedSurface(graph: journey.graph, playback: playerGate);
    await tester.pumpWidget(MaterialApp(home: surface));
    journey.load();
    await journey.waitForCaption(tester, 0);
    await journey.waitForPostCount(tester, 4);
    final startup = await journey.waitForPublishedFocus(tester, 0);
    await journey.waitForFirstFrame(tester, startup);
    await journey.waitForPlaying(tester, startup);
    await waitForCandidateEvidence(tester);
  }

  Future<void> close() async {
    bodyGate.release();
    await journey.close();
  }
}
