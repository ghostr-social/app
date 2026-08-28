import 'package:ghostr/features/settings/domain/data_usage_level.dart';
import 'package:ndk/ndk.dart';

import 'device_resource_ownership.dart';
import 'progressive_device_origin.dart';
import 'progressive_device_resources.dart';
import 'warp_feed_events.dart';
import 'warp_feed_production_graph.dart';
import 'warp_feed_production_graph_build.dart';
import 'warp_feed_relay.dart';

final class WarpFeedDeviceRuntime {
  WarpFeedDeviceRuntime._({
    required this.resources,
    required this.relay,
    required this.events,
    required this.graph,
  });

  static Future<WarpFeedDeviceRuntime> start({
    int eventCount = 3,
    ProgressiveOriginValidator validator = ProgressiveOriginValidator.none,
    DataUsageLevel dataUsage = DataUsageLevel.balanced,
    ProgressiveOriginPacing pacing =
        const ProgressiveOriginPacing.perResponseDelay(
          Duration(milliseconds: 4),
        ),
  }) async {
    return transferDeviceResourceOwnership(
      acquire: () => ProgressiveDeviceResources.start(
        pacing: pacing,
        validator: validator,
      ),
      build: (resources) =>
          _startWithResources(resources, eventCount, dataUsage),
      release: (resources) => resources.close(),
    );
  }

  static Future<WarpFeedDeviceRuntime> _startWithResources(
    ProgressiveDeviceResources resources,
    int eventCount,
    DataUsageLevel dataUsage,
  ) async {
    final events = await signedWarpFeedEvents(
      resources.origin,
      count: eventCount,
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
