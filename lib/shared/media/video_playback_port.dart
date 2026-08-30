import 'package:flutter/widgets.dart';
import 'package:ghostr/core/media/inline_blurhash.dart';
import 'package:ghostr/core/media/hls_playback_authority.dart';
import 'package:ghostr/core/media/playback_asset_authority.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
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

/// Identity of one mounted feed's movable playback surfaces.
final class VideoPlaybackSurfaceScope {}

final class VideoPlaybackSurfaceRequest {
  const VideoPlaybackSurfaceRequest({
    required this.media,
    this.videoId,
    required this.isActive,
    this.mode = VideoPlaybackMode.normal,
    this.surfaceScope,
    this.playbackDeliveryId,
    this.reservesPreparedDecoder = false,
    this.keepWarmWhenInactive = false,
    this.authority,
    this.hlsAuthority,
    this.onHlsFirstFrameRendered,
    this.progressiveRefresh,
    this.onPlaybackMediaReleased,
    InlineBlurHash? preview,
  }) : _preview = preview;

  final VideoMediaSource media;
  final PlaybackVideoId? videoId;
  final bool isActive;
  final VideoPlaybackMode mode;
  final VideoPlaybackSurfaceScope? surfaceScope;
  final PlaybackDeliveryId? playbackDeliveryId;
  final bool reservesPreparedDecoder;
  final bool keepWarmWhenInactive;
  final PlaybackAssetAuthority? authority;
  final HlsPlaybackAuthority? hlsAuthority;
  final ValueChanged<HlsPlaybackAuthority>? onHlsFirstFrameRendered;
  final ProgressivePlaybackRefreshPort? progressiveRefresh;
  final VoidCallback? onPlaybackMediaReleased;
  final InlineBlurHash? _preview;

  InlineBlurHash? get preview => _preview ?? media.mediaMetadata.blurhash;
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
        surfaceScope: request.surfaceScope,
        playbackDeliveryId: request.playbackDeliveryId,
        reservesPreparedDecoder: true,
        keepWarmWhenInactive: request.keepWarmWhenInactive,
        authority: prepared.authority,
        onPlaybackMediaReleased: request.onPlaybackMediaReleased,
        preview: request.preview,
      );

  final PreparedProgressivePlayback prepared;
}

abstract interface class VideoPlaybackPort {
  Widget buildSurface(VideoPlaybackSurfaceRequest request);
}

abstract interface class VideoPlaybackMemoryPressurePort {
  void reportMemoryPressure();
}
