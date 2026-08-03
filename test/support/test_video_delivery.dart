import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/core/work/retrieval_scheduler.dart';
import 'package:ghostr/features/video_catalog/domain/remote_video_source.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';

/// Builds a [ProductionVideoDelivery] for tests, reusing the remote source
/// for the search and discovery paths.
ProductionVideoDelivery testVideoDelivery({
  required VideoInventoryPort inventory,
  required RemoteVideoSource remoteSource,
  HlsPlaybackGatewayPort? hlsPlaybackGateway,
  VideoPlaybackCapabilities playbackCapabilities =
      VideoPlaybackCapabilities.progressiveOnly,
}) {
  return ProductionVideoDelivery(
    inventory,
    remoteSource,
    searchSource: remoteSource,
    discoverySource: remoteSource,
    scheduler: RetrievalScheduler(maxConcurrent: 2),
    hlsPlaybackGateway: hlsPlaybackGateway,
    playbackCapabilities: playbackCapabilities,
  );
}
