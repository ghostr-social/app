part of 'video_player_playback_port.dart';

bool _isPlayableMedia(VideoMediaSource media) {
  return media.isLocal ||
      media is ProxiedHlsVideoMediaSource ||
      media is ProxiedProgressiveVideoMediaSource;
}

VideoPlayerController _videoPlayerController(
  VideoMediaSource media,
  PlayerPreparationAttemptToken? attemptToken,
) {
  const viewType = VideoViewType.textureView;
  if (media is ProxiedHlsVideoMediaSource) {
    return VideoPlayerController.networkUrl(
      media.playbackUri,
      formatHint: VideoFormat.hls,
      viewType: viewType,
    );
  }
  if (media is ProxiedProgressiveVideoMediaSource) {
    return VideoPlayerController.networkUrl(
      media.playbackUri,
      httpHeaders: _attemptHeaders(attemptToken),
      viewType: viewType,
    );
  }
  return VideoPlayerController.file(File(media.localPath!), viewType: viewType);
}

Map<String, String> _attemptHeaders(PlayerPreparationAttemptToken? token) {
  return token == null ? const {} : {warpPlaybackAttemptHeader: token.value};
}

void _requireVisibleVideo(VideoPlayerController controller) {
  final size = controller.value.size;
  if (!size.width.isFinite ||
      !size.height.isFinite ||
      size.width <= 0 ||
      size.height <= 0) {
    throw const _InvisibleVideoTrack();
  }
}

final class _InvisibleVideoTrack implements Exception {
  const _InvisibleVideoTrack();

  @override
  String toString() => 'Media has no visible video track.';
}
