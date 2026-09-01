import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ndk/ndk.dart';

import 'device_resource_ownership.dart';
import 'progressive_device_resources.dart';
import 'warp_feed_device_options.dart';
import 'warp_feed_event_config.dart';
import 'warp_feed_events.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_production_graph_build.dart';
import 'warp_feed_relay.dart';

export 'warp_feed_event_config.dart';
export 'warp_feed_device_options.dart';

final class WarpFeedDeviceRuntime {
  WarpFeedDeviceRuntime._({
    required this.resources,
    required this.relay,
    required this.events,
    required this.graph,
  });

  static Future<WarpFeedDeviceRuntime> start({
    WarpFeedDeviceOptions options = const WarpFeedDeviceOptions(),
  }) async {
    return transferDeviceResourceOwnership(
      acquire: () => ProgressiveDeviceResources.start(
        pacing: options.origin.pacing,
        validator: options.origin.validator,
        rangeSemanticsById: options.origin.rangeSemanticsById,
      ),
      build: (resources) =>
          _startWithResources(resources, options.events, options.dataUsage),
      release: (resources) => resources.close(),
    );
  }

  static Future<WarpFeedDeviceRuntime> _startWithResources(
    ProgressiveDeviceResources resources,
    SignedWarpFeedConfig eventConfig,
    DataUsageLevel dataUsage,
  ) async {
    final events = await signedWarpFeedEvents(
      resources.origin,
      config: eventConfig,
    );
    final relay = await WarpFeedRelay.start(events);
    return _build(resources, relay, events, dataUsage);
  }

  static Future<WarpFeedDeviceRuntime> _build(
    ProgressiveDeviceResources resources,
    WarpFeedRelay relay,
    List<Nip01Event> events,
    DataUsageLevel dataUsage,
  ) async {
    try {
      final graph = await buildWarpFeedProductionGraph(
        resources,
        relay,
        dataUsage,
      );
      return WarpFeedDeviceRuntime._(
        resources: resources,
        relay: relay,
        events: events,
        graph: graph,
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
