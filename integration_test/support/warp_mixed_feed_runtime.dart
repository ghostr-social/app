import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ndk/ndk.dart';

import 'progressive_device_resources.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_production_graph_build.dart';
import 'warp_feed_relay.dart';
import 'warp_hls_playback_gateway_probe.dart';
import 'warp_mixed_feed_events.dart';

final class WarpMixedFeedRuntime {
  WarpMixedFeedRuntime._(
    this.resources,
    this.relay,
    this.events,
    this.graph,
    this.hlsGateway,
  );

  static Future<WarpMixedFeedRuntime> start() async {
    final resources = await ProgressiveDeviceResources.start();
    try {
      return await _startWithResources(resources);
    } on Object {
      await resources.close();
      rethrow;
    }
  }

  static Future<WarpMixedFeedRuntime> _startWithResources(
    ProgressiveDeviceResources resources,
  ) async {
    final events = await signedWarpMixedFeedEvents(resources.origin);
    final relay = await WarpFeedRelay.start(events);
    final hlsGateway = WarpHlsPlaybackGatewayProbe();
    try {
      final graph = await buildWarpFeedProductionGraph(
        resources,
        relay,
        DataUsageLevel.aggressive,
        playbackCapabilities: VideoPlaybackCapabilities.progressiveAndHls,
        hlsPlaybackGateway: hlsGateway,
      );
      return WarpMixedFeedRuntime._(
        resources,
        relay,
        events,
        graph,
        hlsGateway,
      );
    } on Object {
      await relay.close();
      rethrow;
    }
  }

  final ProgressiveDeviceResources resources;
  final WarpFeedRelay relay;
  final List<Nip01Event> events;
  final WarpFeedProductionGraph graph;
  final WarpHlsPlaybackGatewayProbe hlsGateway;
  var _closed = false;

  Future<void> close() async {
    if (_closed) return;
    _closed = true;
    try {
      await graph.close();
    } finally {
      await _closeFixtures();
    }
  }

  Future<void> _closeFixtures() async {
    try {
      await relay.close();
    } finally {
      await resources.close();
    }
  }
}
