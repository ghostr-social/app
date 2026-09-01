import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/features/video_catalog/domain/feed_focus_port.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ndk/ndk.dart';
import 'package:video_player/video_player.dart';

import 'device_playback_probe.dart';
import 'device_qoe_targets.dart';
import 'progressive_device_origin.dart';
import 'progressive_device_resources.dart';
import 'progressive_mp4_fixture.dart';
import 'warp_cancellation_decision_evidence.dart';
import 'warp_feed_player_stage_probe.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_production_graph_build.dart';
import 'warp_feed_relay.dart';
import 'warp_feed_surface.dart';
import 'warp_evidence_models.dart';
import 'warp_focus_plan_timeout_evidence.dart';

part 'warp_long_session_scenario_assertions.dart';
part 'warp_long_session_scenario_burst.dart';
part 'warp_long_session_scenario_correlation.dart';
part 'warp_long_session_scenario_decision.dart';
part 'warp_long_session_scenario_decoded_evidence.dart';
part 'warp_long_session_scenario_driver.dart';
part 'warp_long_session_scenario_events.dart';
part 'warp_long_session_scenario_origin.dart';
part 'warp_long_session_scenario_quiescence.dart';
part 'warp_long_session_scenario_settlement.dart';
part 'warp_long_session_scenario_swipes.dart';
part 'warp_long_session_scenario_wait.dart';

final class WarpLongSessionScenario {
  const WarpLongSessionScenario._(
    this.resources,
    this.relay,
    this.events,
    this.graph,
  );

  static Future<WarpLongSessionScenario> start() async {
    final resources = await ProgressiveDeviceResources.start(
      validator: ProgressiveOriginValidator.stableStrong,
      pacing: const ProgressiveOriginPacing.sharedBandwidth(2500),
    );
    try {
      return await _startWithResources(resources);
    } on Object {
      await resources.close();
      rethrow;
    }
  }

  static Future<WarpLongSessionScenario> _startWithResources(
    ProgressiveDeviceResources resources,
  ) async {
    final events = await _longSessionEvents(resources.origin);
    final relay = await WarpFeedRelay.start(events);
    try {
      final graph = await buildWarpFeedProductionGraph(
        resources,
        relay,
        DataUsageLevel.aggressive,
      );
      return WarpLongSessionScenario._(resources, relay, events, graph);
    } on Object {
      await relay.close();
      rethrow;
    }
  }

  final ProgressiveDeviceResources resources;
  final WarpFeedRelay relay;
  final List<Nip01Event> events;
  final WarpFeedProductionGraph graph;

  Future<void> run(WidgetTester tester) =>
      _WarpLongSessionDriver(this, tester).run();

  Future<void> close() async {
    try {
      await graph.close();
    } finally {
      try {
        await relay.close();
      } finally {
        await resources.close();
      }
    }
  }
}
