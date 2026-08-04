import 'package:ghostr/app/feed_pipeline_flag.dart';
import 'package:ghostr/app/production_video_delivery_infrastructure.dart';
import 'package:ghostr/app/production_video_delivery_sources.dart';
import 'package:ghostr/app/remote_video_delivery_source.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/core/work/retrieval_scheduler.dart';
import 'package:ghostr/features/settings/domain/app_settings.dart';
import 'package:ghostr/features/video_catalog/data/ffi_rust_feed_port.dart';
import 'package:ghostr/features/video_catalog/data/ndk_video_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/playable_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/data/rust_feed_remote_source.dart';
import 'package:ghostr/features/video_catalog/data/scheduled_remote_video_source.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/platform/media/cache_directory_provider.dart';
import 'package:ghostr/platform/media/ffi_hls_playback_gateway.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';
import 'package:ghostr/platform/media/video_player_playback_capabilities.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_outbox_directory.dart';
import 'package:ghostr/platform/nostr/ndk_nostr_video_event_query.dart';
import 'package:ndk/ndk.dart';
import 'package:path_provider/path_provider.dart';

class ProductionVideoDelivery {
  const ProductionVideoDelivery(
    this.remoteSource, {
    required this.searchSource,
    required this.discoverySource,
    required this.scheduler,
    this.hlsPlaybackGateway,
    this.playbackCapabilities = VideoPlaybackCapabilities.progressiveOnly,
  });

  factory ProductionVideoDelivery.disabled() {
    const source = DisabledRemoteVideoSource(
      'Video playback is unavailable on this platform.',
    );
    return ProductionVideoDelivery(
      source,
      searchSource: source,
      discoverySource: source,
      scheduler: RetrievalScheduler(maxConcurrent: 1),
      playbackCapabilities: VideoPlaybackCapabilities.none,
    );
  }

  final RemoteVideoSource remoteSource;

  /// Lean relay path for search: no local merge, prefetch, or fallback.
  final RemoteVideoSource searchSource;

  /// Same lean path, unscheduled, for callers that queue work themselves.
  final RemoteVideoSource discoverySource;

  /// The one queue every content request in the app flows through.
  final RetrievalScheduler scheduler;

  final HlsPlaybackGatewayPort? hlsPlaybackGateway;
  final VideoPlaybackCapabilities playbackCapabilities;
}

/// Builds the Rust discovery pipeline for one viewer. Kept a function
/// so ndk mode never touches the engine's feed FFI.
typedef RustFeedSourceFactory = RemoteVideoSource Function(
  RustFeedViewer viewer,
);

class ProductionVideoDeliveryEnvironment {
  const ProductionVideoDeliveryEnvironment({
    required this.canonicalSource,
    required this.supportDirectoryProvider,
    required this.gateway,
    this.hlsPlaybackGateway = const FfiHlsPlaybackGateway(),
    this.playbackCapabilities = VideoPlaybackCapabilities.progressiveOnly,
    this.feedFlag = const FeedPipelineFlag(),
    this.viewer = noSignedInViewer,
    this.rustFeedSourceBuilder = buildRustFeedSource,
  });

  factory ProductionVideoDeliveryEnvironment.production(
    Ndk ndk,
    AppSettings settings,
    RustFeedViewer viewer,
  ) {
    return ProductionVideoDeliveryEnvironment(
      canonicalSource: NdkVideoRemoteSource(
        NdkNostrVideoEventQuery(
          ndk,
          searchRelays: settings.searchRelays,
          outbox: NdkNostrOutboxDirectory(
            ndk,
            bootstrapRelays: settings.relays,
            maxOutboxRelays: settings.dataUsage.maxOutboxRelays,
          ),
        ),
      ),
      supportDirectoryProvider: getApplicationSupportDirectory,
      gateway: FfiVideoGateway(),
      playbackCapabilities: currentVideoPlayerPlaybackCapabilities(),
      viewer: viewer,
    );
  }

  final RemoteVideoSource canonicalSource;
  final CacheDirectoryProvider supportDirectoryProvider;
  final FfiVideoGateway gateway;
  final HlsPlaybackGatewayPort hlsPlaybackGateway;
  final VideoPlaybackCapabilities playbackCapabilities;

  /// Which discovery pipeline every feed path is built on (plan §5
  /// step 6). ndk until Rust reaches parity.
  final FeedPipelineFlag feedFlag;

  /// The account the viewer-scoped feeds belong to, re-read per request
  /// so signing out — or in as another key — never keeps serving the
  /// previous viewer's feed.
  final RustFeedViewer viewer;
  final RustFeedSourceFactory rustFeedSourceBuilder;
}

/// The Rust discovery pipeline over the generated feed FFI. Built only
/// when [FeedPipelineFlag] asks for it.
RemoteVideoSource buildRustFeedSource(RustFeedViewer viewer) {
  return RustFeedRemoteSource(port: const FfiRustFeedPort(), viewer: viewer);
}

Future<ProductionVideoDelivery> buildProductionVideoDelivery(
  AppSettings settings,
  ProductionVideoDeliveryEnvironment environment,
) async {
  if (!environment.playbackCapabilities.supportsAny) {
    return ProductionVideoDelivery.disabled();
  }
  final gatewayResult = await initializeProductionVideoDeliveryInfrastructure(
    settings: settings,
    directoryProvider: environment.supportDirectoryProvider,
    gateway: environment.gateway,
  );
  final scheduler = RetrievalScheduler(
    maxConcurrent: settings.dataUsage.maxConcurrentRequests,
  );
  logVideoGatewayFailure(gatewayResult);
  final hlsGateway = activeHlsGateway(
    result: gatewayResult,
    gateway: environment.hlsPlaybackGateway,
    capabilities: environment.playbackCapabilities,
  );
  final capabilities = hlsGateway == null
      ? environment.playbackCapabilities.without(VideoMediaDelivery.hls)
      : environment.playbackCapabilities;
  // Downloads follow the viewer's focus window (FeedFocusPort → Rust);
  // the retired native fallback no longer shadows the relay feed.
  final canonical = environment.feedFlag.select(
    ndk: environment.canonicalSource,
    rust: () => environment.rustFeedSourceBuilder(environment.viewer),
  );
  final lean = _playable(canonical, capabilities);
  final source = buildRemoteVideoDeliverySource(primary: lean);
  return ProductionVideoDelivery(
    ScheduledRemoteVideoSource(source: source, scheduler: scheduler),
    searchSource: ScheduledRemoteVideoSource(source: lean, scheduler: scheduler),
    discoverySource: lean,
    scheduler: scheduler,
    hlsPlaybackGateway: hlsGateway,
    playbackCapabilities: capabilities,
  );
}

PlayableRemoteVideoSource _playable(
  RemoteVideoSource source,
  VideoPlaybackCapabilities capabilities,
) =>
    PlayableRemoteVideoSource(source: source, capabilities: capabilities);
