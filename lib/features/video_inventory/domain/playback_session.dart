import 'package:ghostr/core/media/playback_video_id.dart';

final class PlaybackSession {
  factory PlaybackSession(PlaybackVideoId videoId, int generation) {
    if (generation < 1) {
      throw ArgumentError.value(generation, 'generation');
    }
    return PlaybackSession._(videoId, generation);
  }

  const PlaybackSession._(this.videoId, this.generation);

  final PlaybackVideoId videoId;
  final int generation;

  @override
  bool operator ==(Object other) {
    return other is PlaybackSession &&
        other.videoId == videoId &&
        other.generation == generation;
  }

  @override
  int get hashCode => Object.hash(videoId, generation);
}
