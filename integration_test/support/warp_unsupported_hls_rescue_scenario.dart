import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:video_player/video_player.dart';

import 'device_playback_probe.dart';
import 'progressive_device_origin.dart';
import 'warp_feed_player_stage_probe.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_surface.dart';
import 'warp_unsupported_hls_rescue_runtime.dart';

part 'warp_unsupported_hls_rescue_scenario_assertions.dart';
part 'warp_unsupported_hls_rescue_scenario_bounded.dart';
part 'warp_unsupported_hls_rescue_scenario_driver.dart';
part 'warp_unsupported_hls_rescue_scenario_wait.dart';

Future<void> runWarpUnsupportedHlsRescueScenario(WidgetTester tester) async {
  final runtime = await WarpUnsupportedHlsRescueRuntime.start();
  addTearDown(runtime.close);
  await _WarpUnsupportedHlsRescueDriver(runtime, tester).run();
}

typedef _UnsupportedHlsEvidence = ({
  PlaybackFocus failedFocus,
  VideoDeliverySnapshot failure,
  PlaybackFocus rescueFocus,
  Duration before,
  Duration after,
});
