part of 'video_player_playback_port.dart';

bool _isPlayableMedia(VideoMediaSource media) {
  return media.isLocal ||
      media is ProxiedHlsVideoMediaSource ||
      media is ProxiedProgressiveVideoMediaSource;
}

VideoPlayerController _videoPlayerController(VideoMediaSource media) {
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
      viewType: viewType,
    );
  }
  return VideoPlayerController.file(File(media.localPath!), viewType: viewType);
}

void _requireVisibleVideo(VideoPlayerController controller) {
  final size = controller.value.size;
  if (!size.width.isFinite ||
      !size.height.isFinite ||
      size.width <= 0 ||
      size.height <= 0) {
    throw StateError('Media has no visible video track.');
  }
}
