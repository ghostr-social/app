import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_repository.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_cubit.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';
import 'package:ndk/ndk.dart';

import 'device_playback_probe.dart';
import 'progressive_device_origin.dart';
import 'progressive_device_resources.dart';
import 'progressive_device_telemetry.dart';
import 'warp_feed_events.dart';
import 'warp_feed_focus_probe.dart';
import 'warp_feed_player_stage_probe.dart';
import 'warp_feed_preparation_probe.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_production_graph_build.dart';
import 'warp_feed_relay.dart';

part 'warp_feed_playback_journey_ui.dart';
part 'warp_feed_playback_journey_report.dart';
part 'warp_feed_playback_journey_wait.dart';

final class WarpFeedPlaybackJourney {
  WarpFeedPlaybackJourney._({
    required this.resources,
    required this.relay,
    required this.events,
    required this.graph,
  });

  static Future<WarpFeedPlaybackJourney> start() async {
    final resources = await ProgressiveDeviceResources.start(
      responseChunkDelay: const Duration(milliseconds: 4),
    );
    final events = await signedWarpFeedEvents(resources.origin);
    final relay = await WarpFeedRelay.start(events);
    try {
      final graph = await buildWarpFeedProductionGraph(resources, relay);
      return WarpFeedPlaybackJourney._(
        resources: resources,
        relay: relay,
        events: events,
        graph: graph,
      );
    } on Object {
      await relay.close();
      await resources.close();
      rethrow;
    }
  }

  final ProgressiveDeviceResources resources;
  final WarpFeedRelay relay;
  final List<Nip01Event> events;
  final WarpFeedProductionGraph graph;

  FeedCubit get cubit => graph.cubit;
  VideoPlaybackPort get playback => graph.playback;
  ProgressiveDeviceTelemetry get telemetry => graph.telemetry;
  WarpFeedPlayerStageProbe get playerStages => graph.playerStages;
  WarpFeedPreparationMetrics get preparation => graph.preparation;
  WarpFeedFocusProbe get focus => graph.focus;
  VideoFeedRepository get feedRepository => graph.feedRepository;

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
