part of 'audited_video_player_platform.dart';

final class _PlayerState {
  var isPlaying = false;
  var volume = 1.0;
}

final _initialized = VideoEvent(
  eventType: VideoEventType.initialized,
  size: const Size(180, 320),
  duration: const Duration(seconds: 10),
);
