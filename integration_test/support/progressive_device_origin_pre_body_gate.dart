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
  bool get isPeerClosed => _peerClosed.isCompleted;
  bool get timedOut => _timedOut;

  Future<void> _beforeFirstBody(ProgressiveOriginRequest request) async {
    if (!_matches(request)) return;
    if (!_reached.isCompleted) {
      _watchdog = Timer(_timeout, _failOpen);
      _reached.complete();
    }
    await Future.any<void>([_released.future, request._peerClosed.future]);
    if (request.isPeerClosed) {
      throw const ProgressiveOriginPeerClosedBeforeRelease();
    }
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

const _peerClosedBeforeReleaseMessage =
    'Progressive origin peer closed before the pre-body gate released.';

/// The held client dropped its connection before the gate released, so the
/// origin serves it nothing and records the request as client-cancelled.
final class ProgressiveOriginPeerClosedBeforeRelease implements Exception {
  const ProgressiveOriginPeerClosedBeforeRelease();

  @override
  String toString() => _peerClosedBeforeReleaseMessage;
}
