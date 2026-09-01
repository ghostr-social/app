import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/platform/media/ffi_progressive_playback_gateway.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ndk/ndk.dart';
import 'package:video_player/video_player.dart';

import 'device_playback_probe.dart';
import 'progressive_device_resources.dart';
import 'warp_feed_player_stage_probe.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_production_graph_build.dart';
import 'warp_feed_relay.dart';
import 'warp_feed_surface.dart';
import 'warp_validator_rotation_events.dart';
import 'warp_validator_rotation_fixture.dart';

part 'warp_stale_validator_rotation_scenario_assertions.dart';
part 'warp_stale_validator_rotation_scenario_driver.dart';
part 'warp_stale_validator_rotation_scenario_gateway.dart';
part 'warp_stale_validator_rotation_scenario_report.dart';
part 'warp_stale_validator_rotation_scenario_start.dart';
part 'warp_stale_validator_rotation_scenario_wait.dart';

Future<void> runWarpStaleValidatorRotationScenario(WidgetTester tester) async {
  final scenario = await _WarpValidatorRotationScenario.start();
  addTearDown(scenario.close);
  await _WarpValidatorRotationDriver(scenario, tester).run();
}

final class _WarpValidatorRotationScenario {
  _WarpValidatorRotationScenario(_WarpValidatorRotationData data)
    : fixture = data.fixture,
      resources = data.resources,
      relay = data.relay,
      events = data.events,
      graph = data.graph;

  static Future<_WarpValidatorRotationScenario> start() async {
    final fixture = await WarpValidatorRotationFixture.start();
    try {
      return await _startWithFixture(fixture);
    } on Object {
      await fixture.close();
      rethrow;
    }
  }

  final WarpValidatorRotationFixture fixture;
  final ProgressiveDeviceResources resources;
  final WarpFeedRelay relay;
  final List<Nip01Event> events;
  final WarpFeedProductionGraph graph;

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
          await fixture.close();
        }
      }
    }
  }
}

typedef _WarpValidatorRotationData = ({
  WarpValidatorRotationFixture fixture,
  ProgressiveDeviceResources resources,
  WarpFeedRelay relay,
  List<Nip01Event> events,
  WarpFeedProductionGraph graph,
});
