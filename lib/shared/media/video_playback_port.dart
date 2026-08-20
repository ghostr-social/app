import 'package:flutter/widgets.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/core/media/prepared_progressive_playback.dart';
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
    this.authority,
    this.progressiveRefresh,
    this.onPlaybackMediaReleased,
  });

  final VideoMediaSource media;
  final PlaybackVideoId? videoId;
  final bool isActive;
  final VideoPlaybackMode mode;
  final PlaybackAssetAuthority? authority;
  final ProgressivePlaybackRefreshPort? progressiveRefresh;
  final VoidCallback? onPlaybackMediaReleased;
}

final class PreparedProgressiveVideoPlaybackRequest
    extends VideoPlaybackSurfaceRequest {
  factory PreparedProgressiveVideoPlaybackRequest({
    required VideoPlaybackSurfaceRequest request,
    required PreparedProgressivePlayback prepared,
  }) {
    if (request is PreparedProgressiveVideoPlaybackRequest ||
        request.authority != null ||
        request.progressiveRefresh != null ||
        !prepared.matches(request.media)) {
      throw ArgumentError.value(request, 'request', 'Conflicting playback.');
    }
    return PreparedProgressiveVideoPlaybackRequest._(request, prepared);
  }

  PreparedProgressiveVideoPlaybackRequest._(
    VideoPlaybackSurfaceRequest request,
    this.prepared,
  ) : super(
        media: request.media,
        videoId: request.videoId,
        isActive: request.isActive,
        mode: request.mode,
        authority: prepared.authority,
        onPlaybackMediaReleased: request.onPlaybackMediaReleased,
      );

  final PreparedProgressivePlayback prepared;
}

abstract interface class VideoPlaybackPort {
  Widget buildSurface(VideoPlaybackSurfaceRequest request);
}

abstract interface class VideoPlaybackMemoryPressurePort {
  void reportMemoryPressure();
}
