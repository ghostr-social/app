part of 'video_player_playback_port.dart';

bool _isPlayableMedia(VideoMediaSource media) {
  return media.isLocal ||
      media is ProxiedHlsVideoMediaSource ||
      media is ProxiedProgressiveVideoMediaSource;
}

VideoPlayerController _videoPlayerController(VideoMediaSource media) {
  if (media is ProxiedHlsVideoMediaSource) {
    return VideoPlayerController.networkUrl(
      media.playbackUri,
      formatHint: VideoFormat.hls,
    );
  }
  if (media is ProxiedProgressiveVideoMediaSource) {
    return VideoPlayerController.networkUrl(media.playbackUri);
  }
  return VideoPlayerController.file(File(media.localPath!));
}
