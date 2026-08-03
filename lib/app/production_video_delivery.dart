import 'dart:developer';

import 'package:ghostr/app/production_video_delivery_infrastructure.dart';
import 'package:ghostr/app/remote_video_delivery_source.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/ndk_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_snapshot.dart';
import 'package:ghostr/features/video_catalog/data/playable_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/data/remembering_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_inventory/data/inventory_remote_video_source.dart';
import 'package:ghostr/features/video_inventory/domain/disabled_video_inventory.dart';
import 'package:ghostr/features/video_inventory/domain/video_file_downloader.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/platform/media/cache_directory_provider.dart';
import 'package:ghostr/platform/media/ffi_hls_playback_gateway.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/media/video_player_playback_capabilities.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:ndk/ndk.dart';
import 'package:path_provider/path_provider.dart';

class ProductionVideoDelivery {
  const ProductionVideoDelivery(
    this.inventory,
    this.remoteSource, {
    this.hlsPlaybackGateway,
    this.playbackCapabilities = VideoPlaybackCapabilities.progressiveOnly,
  });

  const ProductionVideoDelivery.disabled()
      : inventory = const DisabledVideoInventory(),
        remoteSource = const DisabledRemoteVideoSource(
          'Video playback is unavailable on this platform.',
        ),
        hlsPlaybackGateway = null,
        playbackCapabilities = VideoPlaybackCapabilities.none;

  final VideoInventoryPort inventory;
  final RemoteVideoSource remoteSource;
  final HlsPlaybackGatewayPort? hlsPlaybackGateway;
  final VideoPlaybackCapabilities playbackCapabilities;
}

class ProductionVideoDeliveryEnvironment {
  const ProductionVideoDeliveryEnvironment({
    required this.canonicalSource,
    required this.supportDirectoryProvider,
    required this.downloader,
    required this.gateway,
    this.hlsPlaybackGateway = const FfiHlsPlaybackGateway(),
    this.playbackCapabilities = VideoPlaybackCapabilities.progressiveOnly,
  });

  factory ProductionVideoDeliveryEnvironment.production(Ndk ndk) {
    final mediaPolicy = PublicMediaAddressResolver();
    return ProductionVideoDeliveryEnvironment(
      canonicalSource: NdkVideoRemoteSource(NdkNostrVideoEventQuery(ndk)),
      supportDirectoryProvider: getApplicationSupportDirectory,
      downloader: HttpVideoFileDownloader(
        createPublicMediaHttpClient(mediaPolicy),
        mediaPolicy,
      ),
      gateway: FfiVideoGateway(),
      playbackCapabilities: currentVideoPlayerPlaybackCapabilities(),
    );
  }

  final RemoteVideoSource canonicalSource;
  final CacheDirectoryProvider supportDirectoryProvider;
  final VideoFileDownloader downloader;
  final FfiVideoGateway gateway;
  final HlsPlaybackGatewayPort hlsPlaybackGateway;
  final VideoPlaybackCapabilities playbackCapabilities;
}

Future<ProductionVideoDelivery> buildProductionVideoDelivery(
  AppSettings settings,
  ProductionVideoDeliveryEnvironment environment,
) async {
  if (!environment.playbackCapabilities.supportsAny) {
    return const ProductionVideoDelivery.disabled();
  }
  final infrastructure = await initializeProductionVideoDeliveryInfrastructure(
    settings: settings,
    directoryProvider: environment.supportDirectoryProvider,
    downloader: environment.downloader,
    gateway: environment.gateway,
  );
  final snapshot = NostrVideoSnapshot();
  final canonical = RememberingRemoteVideoSource(
    environment.canonicalSource,
    snapshot,
  );
  final native =
      _nativeRemoteVideoSource(infrastructure.gatewayResult, snapshot);
  final hlsGateway =
      _activeHlsGateway(infrastructure.gatewayResult, environment);
  final capabilities = hlsGateway == null
      ? environment.playbackCapabilities.without(VideoMediaDelivery.hls)
      : environment.playbackCapabilities;
  final source = _inventorySource(
    canonical,
    native,
    infrastructure.inventory,
    capabilities,
  );
  return ProductionVideoDelivery(
    infrastructure.inventory,
    source,
    hlsPlaybackGateway: hlsGateway,
    playbackCapabilities: capabilities,
  );
}

RemoteVideoSource _inventorySource(
  RemoteVideoSource canonical,
  RemoteVideoSource native,
  VideoInventoryPort inventory,
  VideoPlaybackCapabilities capabilities,
) {
  final playablePrimary = _playable(canonical, capabilities);
  final playableFallback = _playable(native, capabilities);
  final combined = buildRemoteVideoDeliverySource(
    primary: playablePrimary,
    nativeFallback: playableFallback,
  );
  return InventoryRemoteVideoSource(source: combined, inventory: inventory);
}

PlayableRemoteVideoSource _playable(
  RemoteVideoSource source,
  VideoPlaybackCapabilities capabilities,
) =>
    PlayableRemoteVideoSource(source: source, capabilities: capabilities);

RemoteVideoSource _nativeRemoteVideoSource(
  VideoGatewayStartResult result,
  NostrVideoSnapshot snapshot,
) {
  return switch (result) {
    VideoGatewayStarted() => FfiVideoRemoteSource(
        snapshotLoader: snapshot.read,
      ),
    VideoGatewayFailed(:final message) => _reportedFailure(message),
  };
}

HlsPlaybackGatewayPort? _activeHlsGateway(
  VideoGatewayStartResult result,
  ProductionVideoDeliveryEnvironment environment,
) {
  if (result is! VideoGatewayStarted ||
      !environment.playbackCapabilities.supportsHls) {
    return null;
  }
  return environment.hlsPlaybackGateway;
}

DisabledRemoteVideoSource _reportedFailure(String message) {
  log(message, name: 'ghostr.gateway');
  return _disabledGateway();
}

DisabledRemoteVideoSource _disabledGateway() {
  return const DisabledRemoteVideoSource(
    'The embedded Nostr gateway is unavailable.',
  );
}
