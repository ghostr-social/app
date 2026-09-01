import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ndk/ndk.dart';

import 'progressive_device_resources.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_production_graph_build.dart';
import 'warp_feed_relay.dart';
import 'warp_hls_playback_gateway_probe.dart';
import 'warp_unsupported_hls_rescue_events.dart';

final class WarpUnsupportedHlsRescueRuntime {
  const WarpUnsupportedHlsRescueRuntime._({
    required this.resources,
    required this.relay,
    required this.events,
    required this.graph,
    required this.hlsGateway,
  });

  static Future<WarpUnsupportedHlsRescueRuntime> start() async {
    final resources = await ProgressiveDeviceResources.start();
    try {
      return await _startWithResources(resources);
    } on Object {
      await resources.close();
      rethrow;
    }
  }

  static Future<WarpUnsupportedHlsRescueRuntime> _startWithResources(
    ProgressiveDeviceResources resources,
  ) async {
    final events = await signedUnsupportedHlsRescueEvents(resources.origin);
    final relay = await WarpFeedRelay.start(events);
    try {
      return await _build(resources, relay, events);
    } on Object {
      await relay.close();
      rethrow;
    }
  }

  static Future<WarpUnsupportedHlsRescueRuntime> _build(
    ProgressiveDeviceResources resources,
    WarpFeedRelay relay,
    List<Nip01Event> events,
  ) async {
    final gateway = WarpHlsPlaybackGatewayProbe();
    final graph = await buildWarpFeedProductionGraph(
      resources,
      relay,
      DataUsageLevel.aggressive,
      playbackCapabilities: VideoPlaybackCapabilities.progressiveAndHls,
      hlsPlaybackGateway: gateway,
    );
    return WarpUnsupportedHlsRescueRuntime._(
      resources: resources,
      relay: relay,
      events: events,
      graph: graph,
      hlsGateway: gateway,
    );
  }

  final ProgressiveDeviceResources resources;
  final WarpFeedRelay relay;
  final List<Nip01Event> events;
  final WarpFeedProductionGraph graph;
  final WarpHlsPlaybackGatewayProbe hlsGateway;

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
