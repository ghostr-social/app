part of 'progressive_device_origin.dart';

final class ProgressiveOriginPreBodyGate {
  ProgressiveOriginPreBodyGate._(Set<String> paths, Duration timeout)
    : _paths = Set.unmodifiable(paths),
      _timeout = timeout {
    if (paths.isEmpty || paths.any((path) => !path.startsWith('/'))) {
      throw ArgumentError.value(paths);
    }
    if (timeout <= Duration.zero) throw ArgumentError.value(timeout);
  }

  final Set<String> _paths;
  final Duration _timeout;
  final _reached = Completer<void>();
  final _released = Completer<void>();
  final _peerClosed = Completer<void>();
  Timer? _watchdog;
  var _timedOut = false;

  Future<void> get reached => _reached.future;
  Future<void> get peerClosed => _peerClosed.future;
  bool get isReached => _reached.isCompleted;
  bool get isReleased => _released.isCompleted;
  bool get timedOut => _timedOut;

  Future<void> _beforeFirstBody(ProgressiveOriginRequest request) async {
    if (!_matches(request)) return;
    if (!_reached.isCompleted) {
      _watchdog = Timer(_timeout, _failOpen);
      _reached.complete();
    }
    await _released.future;
  }

  bool _matches(ProgressiveOriginRequest request) =>
      !isReleased && _tracks(request);

  bool _tracks(ProgressiveOriginRequest request) =>
      request.servedBytes == 0 && _paths.contains(request.path);

  void _observePeerClosed(ProgressiveOriginRequest request) {
    if (_tracks(request) && !_peerClosed.isCompleted) _peerClosed.complete();
  }

  void release() {
    _watchdog?.cancel();
    if (!_released.isCompleted) _released.complete();
  }

  void _failOpen() {
    _timedOut = true;
    release();
  }
}
