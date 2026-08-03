import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/time/clock.dart';

typedef ElapsedClock = Duration Function();

final Stopwatch _systemElapsed = Stopwatch()..start();

Duration _systemElapsedClock() => _systemElapsed.elapsed;

class VideoCacheStoreTiming {
  const VideoCacheStoreTiming({
    this.accessClock = systemClock,
    this.elapsedClock = _systemElapsedClock,
    this.sourceSetTimeout = const Duration(minutes: 5),
  });

  final Clock accessClock;
  final ElapsedClock elapsedClock;
  final Duration sourceSetTimeout;

  VideoCacheDeadline startSourceSet() {
    return VideoCacheDeadline(
      startedAt: elapsedClock(),
      timeout: sourceSetTimeout,
      elapsedClock: elapsedClock,
    );
  }
}

class VideoCacheDeadline {
  const VideoCacheDeadline({
    required this.startedAt,
    required this.timeout,
    required this.elapsedClock,
  });

  final Duration startedAt;
  final Duration timeout;
  final ElapsedClock elapsedClock;

  Duration get remaining {
    final value = timeout - (elapsedClock() - startedAt);
    if (value.inMicroseconds <= 0) {
      throw const VideoCacheSourceSetTimedOut();
    }
    return value;
  }

  void requireActive() => remaining;
}

class VideoCacheSourceSetTimedOut extends AppFailure {
  const VideoCacheSourceSetTimedOut()
      : super('Video cache preparation timed out.');
}
