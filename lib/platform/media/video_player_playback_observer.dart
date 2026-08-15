import 'package:ghostr/features/video_inventory/domain/playback_observation.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';
import 'package:video_player/video_player.dart';

final class VideoPlayerPlaybackObserver {
  bool _hasAdvanced = false;
  Duration _furthestPosition = Duration.zero;

  void reset() {
    _hasAdvanced = false;
    _furthestPosition = Duration.zero;
  }

  PlaybackPhase classify(VideoPlayerValue value, {required bool isActive}) {
    if (!isActive) return PlaybackPhase.inactive;
    if (value.hasError) return PlaybackPhase.failed;
    if (value.isCompleted) return PlaybackPhase.ended;
    _recordProgress(value.position);
    if (value.isPlaying) return _movingPhase;
    return _stoppedPhase(value.isBuffering);
  }

  PlaybackPhase get _movingPhase =>
      _hasAdvanced ? PlaybackPhase.playing : PlaybackPhase.starting;

  PlaybackPhase _stoppedPhase(bool isBuffering) {
    if (!_hasAdvanced) return PlaybackPhase.starting;
    return isBuffering ? PlaybackPhase.networkStalled : PlaybackPhase.paused;
  }

  void _recordProgress(Duration position) {
    if (position <= _furthestPosition) return;
    _hasAdvanced = true;
    _furthestPosition = position;
  }

  PlaybackObservation observe(
    PlaybackSession session,
    VideoPlayerValue value,
    PlaybackPhase phase,
  ) {
    return PlaybackObservation(
      session: session,
      phase: phase,
      metrics: PlaybackMetrics(
        position: value.position,
        bufferedExtent: _bufferedExtent(value),
        playbackRate: value.playbackSpeed,
      ),
    );
  }
}

Duration _bufferedExtent(VideoPlayerValue value) {
  var extent = value.position;
  for (final range in value.buffered) {
    if (range.end < value.position) continue;
    if (range.start > extent) break;
    if (range.end > extent) extent = range.end;
  }
  return extent;
}
