import 'package:flutter/widgets.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/progressive_playback_refresh_port.dart';
import 'package:ghostr/core/media/video_media_source.dart';

enum VideoPlaybackMode {
  normal(shouldPlay: true, speed: 1),
  paused(shouldPlay: false, speed: 1),
  accelerated(shouldPlay: true, speed: 2);

  const VideoPlaybackMode({required this.shouldPlay, required this.speed});

  final bool shouldPlay;
  final double speed;
}

final class VideoPlaybackSurfaceRequest {
  const VideoPlaybackSurfaceRequest({
    required this.media,
    this.videoId,
    required this.isActive,
    this.mode = VideoPlaybackMode.normal,
    this.progressiveRefresh,
    this.onPlaybackMediaReleased,
  });

  final VideoMediaSource media;
  final PlaybackVideoId? videoId;
  final bool isActive;
  final VideoPlaybackMode mode;
  final ProgressivePlaybackRefreshPort? progressiveRefresh;
  final VoidCallback? onPlaybackMediaReleased;
}

abstract interface class VideoPlaybackPort {
  Widget buildSurface(VideoPlaybackSurfaceRequest request);
}
