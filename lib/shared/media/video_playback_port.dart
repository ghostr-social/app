import 'package:flutter/widgets.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/progressive_playback_refresh_port.dart';
import 'package:ghostr/core/media/video_media_source.dart';

final class VideoPlaybackSurfaceRequest {
  const VideoPlaybackSurfaceRequest({
    required this.media,
    this.videoId,
    required this.isActive,
    this.progressiveRefresh,
    this.onPlaybackMediaReleased,
  });

  final VideoMediaSource media;
  final PlaybackVideoId? videoId;
  final bool isActive;
  final ProgressivePlaybackRefreshPort? progressiveRefresh;
  final VoidCallback? onPlaybackMediaReleased;
}

abstract interface class VideoPlaybackPort {
  Widget buildSurface(VideoPlaybackSurfaceRequest request);
}
