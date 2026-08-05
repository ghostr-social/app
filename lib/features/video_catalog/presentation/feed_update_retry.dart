import 'dart:async';

const _defaultDelays = [
  Duration(seconds: 1),
  Duration(seconds: 2),
  Duration(seconds: 4),
  Duration(seconds: 8),
];

/// Bounded reconnect timing for a feed-update stream.
final class FeedUpdateRetry {
  FeedUpdateRetry({List<Duration> delays = _defaultDelays})
    : assert(delays.isNotEmpty),
      _delays = List.unmodifiable(delays);

  final List<Duration> _delays;
  Timer? _timer;
  int _attempt = 0;

  void schedule(void Function() retry) {
    if (_timer != null) return;
    final index = _attempt.clamp(0, _delays.length - 1);
    _attempt += 1;
    _timer = Timer(_delays[index], () {
      _timer = null;
      retry();
    });
  }

  void succeeded() {
    _attempt = 0;
    cancel();
  }

  void reset() {
    _attempt = 0;
    cancel();
  }

  void cancel() {
    _timer?.cancel();
    _timer = null;
  }
}
