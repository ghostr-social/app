import 'package:ghostr/app/production_video_delivery.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/platform/media/inventory_video_playback_port.dart';
import 'package:ghostr/platform/media/unsupported_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

VideoPlaybackPort buildProductionVideoPlayback(
  ProductionVideoDelivery delivery,
) {
  if (!delivery.playbackCapabilities.supportsAny) {
    return const UnsupportedVideoPlaybackPort();
  }
  final inventoryPlayback = InventoryVideoPlaybackPort(
    delegate: const VideoPlayerPlaybackPort(),
    inventory: delivery.inventory,
  );
  final gateway = delivery.hlsPlaybackGateway;
  if (gateway == null || !delivery.playbackCapabilities.supportsHls) {
    return inventoryPlayback;
  }
  return HlsVideoPlaybackPort(
    delegate: inventoryPlayback,
    gateway: gateway,
  );
}
