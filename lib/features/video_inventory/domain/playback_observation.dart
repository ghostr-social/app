import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/features/video_inventory/domain/playback_session.dart';

enum PlaybackPhase {
  starting,
  playing,
  networkStalled,
  paused,
  ended,
  inactive;

  bool get isNetworkStall => this == PlaybackPhase.networkStalled;

  /// Whether this phase advances (or is about to advance) media on screen and
  /// therefore must keep the device screen awake without touch input.
  bool get keepsScreenAwake => switch (this) {
    PlaybackPhase.starting ||
    PlaybackPhase.playing ||
    PlaybackPhase.networkStalled => true,
    PlaybackPhase.paused ||
    PlaybackPhase.ended ||
    PlaybackPhase.inactive => false,
  };
}

final class PlaybackObservation {
  const PlaybackObservation({
    required PlaybackSession session,
    required PlaybackPhase phase,
    required PlaybackMetrics metrics,
  }) : session = session,
       phase = phase,
       metrics = metrics;

  final PlaybackSession session;
  final PlaybackPhase phase;
  final PlaybackMetrics metrics;

  PlaybackVideoId get videoId => session.videoId;

  Duration get position => metrics.position;

  Duration get bufferedExtent => metrics.bufferedExtent;

  double get playbackRate => metrics.playbackRate;

  Duration get bufferAhead => metrics.bufferAhead;
}

final class PlaybackMetrics {
  factory PlaybackMetrics({
    required Duration position,
    required Duration bufferedExtent,
    required double playbackRate,
  }) {
    _validateTimeline(position, bufferedExtent);
    _validateRate(playbackRate);
    return PlaybackMetrics._(position, bufferedExtent, playbackRate);
  }

  const PlaybackMetrics._(
    this.position,
    this.bufferedExtent,
    this.playbackRate,
  );

  final Duration position;
  final Duration bufferedExtent;
  final double playbackRate;

  Duration get bufferAhead => bufferedExtent - position;
}

void _validateTimeline(Duration position, Duration bufferedExtent) {
  if (position.isNegative || bufferedExtent < position) {
    throw ArgumentError('Buffered extent must include playback position.');
  }
}

void _validateRate(double playbackRate) {
  if (!playbackRate.isFinite || playbackRate <= 0) {
    throw ArgumentError.value(playbackRate, 'playbackRate');
  }
}
