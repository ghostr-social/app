import 'dart:io';

import 'package:ghostr/app/build_production_dependencies.dart';
import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

import 'progressive_device_resources.dart';
import 'warp_controlled_network_status.dart';
import 'warp_feed_preparation_probe.dart';
import 'warp_feed_rust_probe.dart';

Future<ProductionVideoDelivery> buildWarpFeedProductionDelivery(
  WarpFeedProductionDeliveryInput input,
) {
  return buildProductionVideoDelivery(
    input.settings,
    ProductionVideoDeliveryEnvironment(
      source: RustFeedRemoteSource(
        port: input.rustProbe,
        viewer: signedInViewer(input.nostr.eventClient),
      ),
      adapters: ProductionVideoDeliveryAdapters(
        supportDirectoryProvider: () async =>
            Directory(input.resources.cachePath),
        gateway: WarpDeviceFfiVideoGateway(input.resources.origin.origin),
        preparationUpdates: input.preparation,
        networkStatus: input.network,
      ),
      playbackCapabilities: VideoPlaybackCapabilities.progressiveOnly,
    ),
  );
}

typedef WarpFeedProductionDeliveryInput = ({
  AppSettings settings,
  ProductionNostrServices nostr,
  ProgressiveDeviceResources resources,
  WarpFeedPreparationProbe preparation,
  WarpFeedRustProbe rustProbe,
  WarpControlledNetworkStatus network,
});

final class WarpDeviceFfiVideoGateway extends FfiVideoGateway {
  WarpDeviceFfiVideoGateway(this.origin);

  final Uri origin;

  @override
  Future<VideoGatewayStartResult> start(
    AppSettings settings,
    String cacheDirectory, {
    Uri? deviceIntegrationOrigin,
    DeliveryNetworkStatus initialNetwork = DeliveryNetworkStatus.unavailable,
  }) {
    return super.start(
      settings,
      cacheDirectory,
      deviceIntegrationOrigin: origin,
      initialNetwork: initialNetwork,
    );
  }
}
