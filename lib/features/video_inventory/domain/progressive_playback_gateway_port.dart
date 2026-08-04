import 'package:ghostr/core/media/video_media_source.dart';

/// Resolves remote progressive media into a loopback URL served by the
/// embedded gateway, which streams bytes as they arrive instead of
/// gating playback behind a full download.
abstract interface class ProgressivePlaybackGatewayPort {
  Future<ProxiedProgressiveVideoMediaSource> resolve(VideoMediaSource media);
}
