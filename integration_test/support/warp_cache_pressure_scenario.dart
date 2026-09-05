import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ghostr/src/rust/api/engine_control.dart';
import 'package:video_player/video_player.dart';

import 'device_playback_probe.dart';
import 'progressive_device_origin.dart';
import 'warp_cache_pressure_storage.dart';
import 'warp_feed_player_stage_probe.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_surface.dart';
import 'warp_long_session_scenario.dart';
import 'warp_native_request_bounds.dart';

part 'warp_cache_pressure_scenario_assertions.dart';
part 'warp_cache_pressure_scenario_diagnostics.dart';
part 'warp_cache_pressure_scenario_configuration.dart';
part 'warp_cache_pressure_scenario_driver.dart';
part 'warp_cache_pressure_scenario_quiescence.dart';
part 'warp_cache_pressure_scenario_storage.dart';
part 'warp_cache_pressure_scenario_swipes.dart';
part 'warp_cache_pressure_scenario_wait.dart';

final class WarpCachePressureScenario {
  const WarpCachePressureScenario._(this.session);

  static Future<WarpCachePressureScenario> start() async {
    final session = await WarpLongSessionScenario.start();
    try {
      await _configureCachePressure(session);
      return WarpCachePressureScenario._(session);
    } on Object {
      await session.close();
      rethrow;
    }
  }

  final WarpLongSessionScenario session;

  Future<void> run(WidgetTester tester) =>
      _WarpCachePressureDriver(this, tester).run();

  Future<void> close() => session.close();
}
