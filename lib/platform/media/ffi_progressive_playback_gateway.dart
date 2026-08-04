import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_inventory/domain/progressive_playback_gateway_port.dart';
import 'package:ghostr/platform/media/ffi_focus_item_media_mapper.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:ghostr/src/rust/api/focus_control.dart';

typedef RustPlaybackUrlResolver = Future<String> Function({
  required FfiFocusItem item,
});

final class FfiProgressivePlaybackGateway
    implements ProgressivePlaybackGatewayPort {
  const FfiProgressivePlaybackGateway({
    RustPlaybackUrlResolver resolvePlaybackUrl = ffiPlaybackUrl,
  }) : _resolvePlaybackUrl = resolvePlaybackUrl;

  final RustPlaybackUrlResolver _resolvePlaybackUrl;

  @override
  Future<ProxiedProgressiveVideoMediaSource> resolve(
    VideoMediaSource media,
  ) async {
    final item = ffiFocusItemForMedia(media);
    final url = await _resolvePlaybackUrl(item: item);
    return ProxiedProgressiveVideoMediaSource(url);
  }
}
