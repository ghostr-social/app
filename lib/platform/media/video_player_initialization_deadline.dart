part of 'video_player_playback_port.dart';

enum _InitializationExit { initialized, closed, superseded, timedOut }

final class _InitializationDeadline {
  _InitializationDeadline(Duration duration) {
    _timer = Timer(duration, () => _complete(_InitializationExit.timedOut));
  }

  final Completer<_InitializationExit> _expired = Completer();
  late final Timer _timer;

  Future<_InitializationExit> wait({
    required Future<void> initialization,
    required Future<void> closed,
    required Future<void> superseded,
  }) async {
    try {
      return await Future.any([
        initialization.then((_) => _InitializationExit.initialized),
        closed.then((_) => _InitializationExit.closed),
        superseded.then((_) => _InitializationExit.superseded),
        _expired.future,
      ]);
    } finally {
      _timer.cancel();
    }
  }

  void _complete(_InitializationExit exit) {
    if (!_expired.isCompleted) _expired.complete(exit);
  }
}
