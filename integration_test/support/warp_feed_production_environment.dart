import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ndk/ndk.dart';
import 'package:sembast/sembast_memory.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'progressive_device_resources.dart';
import 'warp_controlled_network_status.dart';
import 'warp_feed_preparation_probe.dart';
import 'warp_feed_production_delivery.dart';
import 'warp_feed_rust_probe.dart';

final class WarpFeedProductionCapture {
  ProductionNostrServices? nostr;
  ProductionVideoDelivery? delivery;
  final rustProbe = WarpFeedRustProbe();
  final network = WarpControlledNetworkStatus();
}

ProductionDependenciesEnvironment warpFeedProductionEnvironment(
  WarpFeedProductionEnvironmentInput input,
) {
  return ProductionDependenciesEnvironment(
    preferencesLoader: SharedPreferences.getInstance,
    nostrServicesBuilder: (settings) {
      final nostr = buildProductionNostrServices(
        settings,
        ndkBuilder: () => input.ndk,
      );
      input.capture.nostr = nostr;
      return nostr;
    },
    videoDeliveryBuilder: (settings, nostr) async {
      final delivery = await buildWarpFeedProductionDelivery(
        (
          settings: settings,
          nostr: nostr,
          resources: input.resources,
          preparation: input.preparation,
          rustProbe: input.capture.rustProbe,
          network: input.capture.network,
          deviceIntegrationOrigin: input.deviceIntegrationOrigin,
        ),
        playbackCapabilities: input.playbackCapabilities,
        hlsPlaybackGateway: input.hlsPlaybackGateway,
      );
      input.capture.delivery = delivery;
      return delivery;
    },
    watchHistoryDatabaseLoader: () => databaseFactoryMemory.openDatabase(
      'warp-feed-${DateTime.now().microsecondsSinceEpoch}',
      mode: DatabaseMode.create,
    ),
  );
}

typedef WarpFeedProductionEnvironmentInput = ({
  Ndk ndk,
  ProgressiveDeviceResources resources,
  WarpFeedPreparationProbe preparation,
  WarpFeedProductionCapture capture,
  VideoPlaybackCapabilities playbackCapabilities,
  HlsPlaybackGatewayPort? hlsPlaybackGateway,
  Uri? deviceIntegrationOrigin,
});
