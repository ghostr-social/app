import 'package:ghostr/app/production_video_delivery_infrastructure.dart';
import 'package:ghostr/app/production_video_delivery_sources.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/core/network/delivery_network_status.dart';
import 'package:ghostr/core/network/delivery_network_status_port.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/data/ffi_rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/playable_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation_updates.dart';
import 'package:ghostr/platform/media/cache_directory_provider.dart';
import 'package:ghostr/platform/media/delivery_network_status_runtime.dart';
import 'package:ghostr/platform/media/ffi_hls_playback_gateway.dart';
import 'package:ghostr/platform/media/ffi_playback_preparation_updates.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';
import 'package:ghostr/platform/media/video_player_playback_capabilities.dart';
import 'package:ghostr/platform/network/delivery_network_status_platform.dart';
import 'package:ghostr/platform/network/unavailable_delivery_network_status.dart';
import 'package:path_provider/path_provider.dart';

class ProductionVideoDelivery {
  const ProductionVideoDelivery(
    this.sources, {
    this.hlsPlaybackGateway,
    this.preparationUpdates,
    this.playbackCapabilities = VideoPlaybackCapabilities.progressiveOnly,
    this.networkStatusRuntime,
  });

  factory ProductionVideoDelivery.disabled() {
    const source = DisabledRemoteVideoSource(
      'Video playback is unavailable on this platform.',
    );
    return ProductionVideoDelivery(
      ProductionVideoDeliverySources.shared(source),
      playbackCapabilities: VideoPlaybackCapabilities.none,
    );
  }

  final ProductionVideoDeliverySources sources;

  RemoteVideoSource get remoteSource => sources.primary;

  /// Direct Rust feed path for search.
  RemoteVideoSource get searchSource => sources.search;

  /// Direct Rust feed path for background discovery.
  RemoteVideoSource get discoverySource => sources.discovery;

  final HlsPlaybackGatewayPort? hlsPlaybackGateway;
  final PlaybackPreparationUpdates? preparationUpdates;
  final VideoPlaybackCapabilities playbackCapabilities;
  final DeliveryNetworkStatusRuntime? networkStatusRuntime;

  Future<void> dispose() async {
    await networkStatusRuntime?.close();
  }
}

final class ProductionVideoDeliverySources {
  const ProductionVideoDeliverySources.shared(RemoteVideoSource source)
    : primary = source,
      search = source,
      discovery = source;

  final RemoteVideoSource primary;
  final RemoteVideoSource search;
  final RemoteVideoSource discovery;
}

class ProductionVideoDeliveryEnvironment {
  const ProductionVideoDeliveryEnvironment({
    required this.source,
    required this.adapters,
    this.playbackCapabilities = VideoPlaybackCapabilities.progressiveOnly,
  });

  factory ProductionVideoDeliveryEnvironment.production(RustFeedViewer viewer) {
    return ProductionVideoDeliveryEnvironment(
      source: buildRustFeedSource(viewer),
      adapters: ProductionVideoDeliveryAdapters(
        supportDirectoryProvider: getApplicationSupportDirectory,
        gateway: FfiVideoGateway(),
        networkStatus: currentDeliveryNetworkStatusPlatform(),
      ),
      playbackCapabilities: currentVideoPlayerPlaybackCapabilities(),
    );
  }

  final RemoteVideoSource source;
  final ProductionVideoDeliveryAdapters adapters;
  final VideoPlaybackCapabilities playbackCapabilities;
}

final class ProductionVideoDeliveryAdapters {
  const ProductionVideoDeliveryAdapters({
    required this.supportDirectoryProvider,
    required this.gateway,
    this.hlsPlaybackGateway = const FfiHlsPlaybackGateway(),
    this.preparationUpdates = const FfiPlaybackPreparationUpdates(),
    this.networkStatus = const UnavailableDeliveryNetworkStatus(),
    this.applyNetworkStatus = updateRustDeliveryNetworkStatus,
  });

  final CacheDirectoryProvider supportDirectoryProvider;
  final FfiVideoGateway gateway;
  final HlsPlaybackGatewayPort hlsPlaybackGateway;
  final PlaybackPreparationUpdates preparationUpdates;
  final DeliveryNetworkStatusPort networkStatus;
  final DeliveryNetworkStatusApplier applyNetworkStatus;
}

/// The Rust discovery pipeline over the generated feed FFI.
RemoteVideoSource buildRustFeedSource(RustFeedViewer viewer) {
  return RustFeedRemoteSource(port: const FfiRustFeedPort(), viewer: viewer);
}

Future<ProductionVideoDelivery> buildProductionVideoDelivery(
  AppSettings settings,
  ProductionVideoDeliveryEnvironment environment,
) async {
  final networkPort = environment.adapters.networkStatus;
  final initialNetwork = await _initialNetwork(networkPort);
  final gatewayResult = await initializeProductionVideoDeliveryInfrastructure(
    settings: settings,
    directoryProvider: environment.adapters.supportDirectoryProvider,
    gateway: environment.adapters.gateway,
    initialNetwork: initialNetwork,
  );
  if (gatewayResult case VideoGatewayFailed(:final message)) {
    await networkPort.close();
    throw AppFailure(message);
  }
  final networkRuntime = await DeliveryNetworkStatusRuntime.start(
    port: networkPort,
    initial: initialNetwork,
    apply: environment.adapters.applyNetworkStatus,
  );
  if (!environment.playbackCapabilities.supportsAny) {
    await networkRuntime.close();
    return ProductionVideoDelivery.disabled();
  }
  final hlsGateway = activeHlsGateway(
    result: gatewayResult,
    gateway: environment.adapters.hlsPlaybackGateway,
    capabilities: environment.playbackCapabilities,
  );
  final capabilities = hlsGateway == null
      ? environment.playbackCapabilities.without(VideoMediaDelivery.hls)
      : environment.playbackCapabilities;
  final source = _playable(environment.source, capabilities);
  return ProductionVideoDelivery(
    ProductionVideoDeliverySources.shared(source),
    hlsPlaybackGateway: hlsGateway,
    preparationUpdates: capabilities.supportsProgressive
        ? environment.adapters.preparationUpdates
        : null,
    playbackCapabilities: capabilities,
    networkStatusRuntime: networkRuntime,
  );
}

Future<DeliveryNetworkStatus> _initialNetwork(
  DeliveryNetworkStatusPort port,
) async {
  try {
    return await port.read();
  } on Object catch (error, stackTrace) {
    logBoundaryFailure(
      source: 'ghostr.video.network',
      message: 'The initial delivery network class is unavailable.',
      error: error,
      stackTrace: stackTrace,
    );
    return DeliveryNetworkStatus.unavailable;
  }
}

PlayableRemoteVideoSource _playable(
  RemoteVideoSource source,
  VideoPlaybackCapabilities capabilities,
) => PlayableRemoteVideoSource(source: source, capabilities: capabilities);
