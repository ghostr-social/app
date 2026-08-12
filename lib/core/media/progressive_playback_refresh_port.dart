import 'package:ghostr/core/media/video_media_source.dart';

/// Renews the capability for one already-resolved progressive playback source.
abstract interface class ProgressivePlaybackRefreshPort {
  Future<ProxiedProgressiveVideoMediaSource> refresh();
}
