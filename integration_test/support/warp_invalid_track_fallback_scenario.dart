import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/app/production_video_playback.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/features/video_inventory/domain/player_preparation_feedback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ndk/ndk.dart';
import 'package:video_player/video_player.dart';

import 'device_playback_probe.dart';
import 'progressive_device_origin.dart';
import 'progressive_device_resources.dart';
import 'warp_feed_player_stage_probe.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_production_graph_build.dart';
import 'warp_feed_relay.dart';
import 'warp_feed_surface.dart';
import 'warp_invalid_track_fallback_events.dart';
import 'warp_no_video_rendition_fixture.dart';
import 'warp_player_failure_recorder.dart';

part 'warp_invalid_track_fallback_scenario_assertions.dart';
part 'warp_invalid_track_fallback_scenario_bounded.dart';
part 'warp_invalid_track_fallback_scenario_driver.dart';
part 'warp_invalid_track_fallback_scenario_evidence.dart';
part 'warp_invalid_track_fallback_scenario_start.dart';
part 'warp_invalid_track_fallback_scenario_wait.dart';
part 'warp_invalid_track_fallback_scenario_wait_core.dart';

final class WarpInvalidTrackFallbackScenario {
  const WarpInvalidTrackFallbackScenario._(this._runtime);

  static Future<WarpInvalidTrackFallbackScenario> start() {
    return _startInvalidTrackFallbackScenario();
  }

  final _InvalidTrackRuntime _runtime;

  ProgressiveDeviceResources get resources => _runtime.resources;
  WarpNoVideoRenditionFixture get fixture => _runtime.fixture;
  WarpFeedRelay get relay => _runtime.relay;
  List<Nip01Event> get events => _runtime.events;
  WarpFeedProductionGraph get graph => _runtime.graph;
  WarpPlayerFailureRecorder get failures => _runtime.failures;
  VideoPlaybackPort get playback => _runtime.playback;

  Future<void> run(WidgetTester tester) {
    return _WarpInvalidTrackFallbackDriver(this, tester).run();
  }

  Future<void> close() async {
    try {
      await graph.close();
    } finally {
      try {
        await relay.close();
      } finally {
        try {
          await resources.close();
        } finally {
          fixture.restore();
        }
      }
    }
  }
}
