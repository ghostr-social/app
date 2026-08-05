import 'package:ghostr/core/media/video_playback_capabilities.dart';
import 'package:ghostr/features/video_inventory/domain/hls_playback_gateway_port.dart';
import 'package:ghostr/platform/media/ffi_video_gateway.dart';

HlsPlaybackGatewayPort? activeHlsGateway({
  required VideoGatewayStartResult result,
  required HlsPlaybackGatewayPort gateway,
  required VideoPlaybackCapabilities capabilities,
}) {
  if (result is! VideoGatewayStarted || !capabilities.supportsHls) {
    return null;
  }
  return gateway;
}
