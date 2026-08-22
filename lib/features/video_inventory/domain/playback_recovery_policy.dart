const maxAutomaticPlaybackRecoveryAttempts = 4;
const playbackInitializationTimeout = Duration(seconds: 2);
const playbackControllerTeardownTimeout = Duration(seconds: 5);

enum PlaybackSurfaceActivity { active, inactive }

final class PlaybackRecoveryAttempt {
  const PlaybackRecoveryAttempt._(this.index);

  static const first = PlaybackRecoveryAttempt._(0);

  final int index;

  PlaybackRecoveryAttempt get next => PlaybackRecoveryAttempt._(index + 1);

  @override
  bool operator ==(Object other) {
    return other is PlaybackRecoveryAttempt && other.index == index;
  }

  @override
  int get hashCode => index.hashCode;
}

sealed class PlaybackRecoveryDecision {
  const PlaybackRecoveryDecision();
}

final class PlaybackRecoveryScheduled extends PlaybackRecoveryDecision {
  PlaybackRecoveryScheduled(this.delay) {
    if (delay.isNegative) throw ArgumentError.value(delay, 'delay');
  }

  final Duration delay;

  @override
  bool operator ==(Object other) {
    return other is PlaybackRecoveryScheduled && other.delay == delay;
  }

  @override
  int get hashCode => delay.hashCode;
}

final class PlaybackRecoveryDeferred extends PlaybackRecoveryDecision {
  const PlaybackRecoveryDeferred();

  @override
  bool operator ==(Object other) => other is PlaybackRecoveryDeferred;

  @override
  int get hashCode => runtimeType.hashCode;
}

final class PlaybackRecoveryExhausted extends PlaybackRecoveryDecision {
  const PlaybackRecoveryExhausted();

  @override
  bool operator ==(Object other) => other is PlaybackRecoveryExhausted;

  @override
  int get hashCode => runtimeType.hashCode;
}

final class PlaybackRecoveryPolicy {
  factory PlaybackRecoveryPolicy(Iterable<Duration> retryDelays) {
    final delays = List<Duration>.unmodifiable(retryDelays);
    _validateSchedule(delays);
    return PlaybackRecoveryPolicy._(delays);
  }

  const PlaybackRecoveryPolicy.standard()
    : _retryDelays = const [
        Duration.zero,
        Duration(milliseconds: 250),
        Duration(seconds: 1),
      ];

  const PlaybackRecoveryPolicy.disabled() : _retryDelays = const [];

  const PlaybackRecoveryPolicy._(this._retryDelays);

  final List<Duration> _retryDelays;

  Duration get initializationTimeout => playbackInitializationTimeout;

  Duration get teardownTimeout => playbackControllerTeardownTimeout;

  PlaybackRecoveryDecision decide(
    PlaybackRecoveryAttempt attempt,
    PlaybackSurfaceActivity activity,
  ) {
    if (_retryDelays.isEmpty) return const PlaybackRecoveryExhausted();
    if (activity == PlaybackSurfaceActivity.inactive) {
      return const PlaybackRecoveryDeferred();
    }
    if (attempt.index >= _retryDelays.length) {
      return const PlaybackRecoveryExhausted();
    }
    return PlaybackRecoveryScheduled(_retryDelays[attempt.index]);
  }
}

final class PlaybackResumePoint {
  factory PlaybackResumePoint(Duration position) {
    if (position.isNegative) {
      throw ArgumentError.value(position, 'position');
    }
    return PlaybackResumePoint._(position);
  }

  const PlaybackResumePoint._(this.position);

  static const start = PlaybackResumePoint._(Duration.zero);

  final Duration position;

  Duration within(Duration duration) {
    if (duration.isNegative) throw ArgumentError.value(duration, 'duration');
    return position > duration ? duration : position;
  }
}

void _validateSchedule(List<Duration> delays) {
  if (delays.isEmpty || delays.length > maxAutomaticPlaybackRecoveryAttempts) {
    throw ArgumentError.value(delays, 'retryDelays');
  }
  if (delays.any((delay) => delay.isNegative)) {
    throw ArgumentError.value(delays, 'retryDelays');
  }
}
