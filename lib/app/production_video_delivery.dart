import 'package:ghostr/app/production_video_delivery_infrastructure.dart';
import 'package:ghostr/app/production_video_delivery_sources.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/data/ffi_rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/playable_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/platform/media/cache_directory_provider.dart';
import 'package:ghostr/platform/media/ffi_hls_playback_gateway.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';
import 'package:ghostr/platform/media/video_player_playback_capabilities.dart';
import 'package:path_provider/path_provider.dart';

class ProductionVideoDelivery {
  const ProductionVideoDelivery(
    this.sources, {
    this.hlsPlaybackGateway,
    this.playbackCapabilities = VideoPlaybackCapabilities.progressiveOnly,
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
  final VideoPlaybackCapabilities playbackCapabilities;
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

  factory ProductionVideoDeliveryEnvironment.production(
    RustFeedViewer viewer,
  ) {
    return ProductionVideoDeliveryEnvironment(
      source: buildRustFeedSource(viewer),
      adapters: ProductionVideoDeliveryAdapters(
        supportDirectoryProvider: getApplicationSupportDirectory,
        gateway: FfiVideoGateway(),
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
  });

  final CacheDirectoryProvider supportDirectoryProvider;
  final FfiVideoGateway gateway;
  final HlsPlaybackGatewayPort hlsPlaybackGateway;
}

/// The Rust discovery pipeline over the generated feed FFI.
RemoteVideoSource buildRustFeedSource(RustFeedViewer viewer) {
  return RustFeedRemoteSource(port: const FfiRustFeedPort(), viewer: viewer);
}

Future<ProductionVideoDelivery> buildProductionVideoDelivery(
  AppSettings settings,
  ProductionVideoDeliveryEnvironment environment,
) async {
  final gatewayResult = await initializeProductionVideoDeliveryInfrastructure(
    settings: settings,
    directoryProvider: environment.adapters.supportDirectoryProvider,
    gateway: environment.adapters.gateway,
  );
  if (gatewayResult case VideoGatewayFailed(:final message)) {
    throw AppFailure(message);
  }
  if (!environment.playbackCapabilities.supportsAny) {
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
    playbackCapabilities: capabilities,
  );
}

PlayableRemoteVideoSource _playable(
  RemoteVideoSource source,
  VideoPlaybackCapabilities capabilities,
) =>
    PlayableRemoteVideoSource(source: source, capabilities: capabilities);
