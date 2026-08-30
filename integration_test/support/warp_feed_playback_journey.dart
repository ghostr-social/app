import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ndk/ndk.dart';

import 'device_playback_probe.dart';
import 'device_qoe_targets.dart';
import 'progressive_device_origin.dart';
import 'progressive_device_resources.dart';
import 'progressive_device_telemetry.dart';
import 'warp_evidence_reader.dart';
import 'warp_evidence_models.dart';
import 'warp_feed_delivery_probe.dart';
import 'warp_feed_device_runtime.dart';
import 'warp_feed_focus_probe.dart';
import 'warp_feed_player_stage_probe.dart';
import 'warp_feed_preparation_probe.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_relay.dart';
import 'warp_feed_surface.dart';
import 'warp_ready_window_acceptance.dart';

part 'warp_feed_playback_journey_ui.dart';
part 'warp_feed_playback_journey_assertions.dart';
part 'warp_feed_playback_journey_assertion_evidence.dart';
part 'warp_feed_playback_journey_playback_contract.dart';
part 'warp_feed_playback_journey_cadence_assertions.dart';
part 'warp_feed_playback_journey_gestures.dart';
part 'warp_feed_playback_journey_report.dart';
part 'warp_feed_playback_journey_network.dart';
part 'warp_feed_playback_journey_paths.dart';
part 'warp_feed_playback_journey_swipe.dart';
part 'warp_feed_playback_journey_recovery_window.dart';
part 'warp_feed_playback_journey_window.dart';
part 'warp_feed_playback_journey_wait.dart';
part 'warp_feed_playback_journey_wait_core.dart';
part 'warp_feed_playback_journey_origin_wait.dart';
part 'warp_feed_playback_journey_origin_assertions.dart';
part 'warp_feed_playback_journey_store_assertions.dart';
part 'warp_feed_playback_journey_link_wait.dart';
part 'warp_feed_playback_journey_decision_wait.dart';
part 'warp_feed_playback_journey_pair_wait.dart';
part 'warp_feed_playback_journey_promotion_wait.dart';
part 'warp_feed_playback_journey_loop_reopen.dart';

final class WarpFeedPlaybackJourney {
  WarpFeedPlaybackJourney._(this.runtime);

  static Future<WarpFeedPlaybackJourney> start({
    int eventCount = 3,
    ProgressiveOriginValidator validator = ProgressiveOriginValidator.none,
    DataUsageLevel dataUsage = DataUsageLevel.balanced,
    ProgressiveOriginPacing pacing =
        const ProgressiveOriginPacing.perResponseDelay(
          Duration(milliseconds: 4),
        ),
  }) async {
    final runtime = await WarpFeedDeviceRuntime.start(
      eventCount: eventCount,
      validator: validator,
      dataUsage: dataUsage,
      pacing: pacing,
    );
    return WarpFeedPlaybackJourney._(runtime);
  }

  final WarpFeedDeviceRuntime runtime;
  final playbackErrorSamples = <Duration>[];

  ProgressiveDeviceResources get resources => runtime.resources;
  WarpFeedRelay get relay => runtime.relay;
  List<Nip01Event> get events => runtime.events;
  WarpFeedProductionGraph get graph => runtime.graph;

  FeedCubit get cubit => graph.cubit;
  VideoPlaybackPort get playback => graph.playback;
  ProgressiveDeviceTelemetry get telemetry => graph.telemetry;
  WarpFeedPlayerStageProbe get playerStages => graph.playerStages;
  WarpFeedPreparationMetrics get preparation => graph.preparation;
  WarpFeedFocusProbe get focus => graph.focus;
  WarpEvidenceReader get evidence => graph.evidence;
  VideoFeedRepository get feedRepository => graph.feedRepository;
  bool get hadPlaybackError => playbackErrorSamples.isNotEmpty;

  Future<void> close() => runtime.close();
}
