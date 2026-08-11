import 'package:flutter/widgets.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/video_media_source.dart';

final class VideoPlaybackSurfaceRequest {
  const VideoPlaybackSurfaceRequest({
    required this.media,
    this.videoId,
    required this.isActive,
    this.onPlaybackMediaReleased,
  });

  final VideoMediaSource media;
  final PlaybackVideoId? videoId;
  final bool isActive;
  final VoidCallback? onPlaybackMediaReleased;
}

abstract interface class VideoPlaybackPort {
  Widget buildSurface({
    required VideoMediaSource media,
    PlaybackVideoId? videoId,
    required bool isActive,
    void Function()? onPlaybackMediaReleased,
  });
}
