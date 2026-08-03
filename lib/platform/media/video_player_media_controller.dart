part of 'video_player_playback_port.dart';

bool _isPlayableMedia(VideoMediaSource media) {
  return media.isLocal || media is ProxiedHlsVideoMediaSource;
}

VideoPlayerController _videoPlayerController(VideoMediaSource media) {
  if (media is ProxiedHlsVideoMediaSource) {
    return VideoPlayerController.networkUrl(
      media.playbackUri,
      formatHint: VideoFormat.hls,
    );
  }
  return VideoPlayerController.file(File(media.localPath!));
}
