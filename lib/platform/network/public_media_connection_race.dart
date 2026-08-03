part of 'public_media_http_client.dart';

abstract interface class _CancelableConnection {
  void cancel();
}

class _MediaConnectionRace<T> implements _CancelableConnection {
  _MediaConnectionRace(
    this._startConnect,
    this._close,
    this._port,
    this._fallbackDelay,
  );

  final _MediaConnectionStarter<T> _startConnect;
  final _MediaConnectionCloser<T> _close;
  final int _port;
  final Duration _fallbackDelay;
  final Completer<T> _result = Completer<T>();
  final List<_MediaSocketAttempt<T>> _attempts = [];
  late final List<InternetAddress> _addresses;
  Timer? _timer;
  Object? _firstError;
  StackTrace? _firstStackTrace;
  var _nextIndex = 0;
  var _activeAttempts = 0;
  var _cancelled = false;

  Future<T> connect(List<InternetAddress> addresses) {
    _addresses = addresses;
    _startNext();
    return _result.future;
  }

  void _startNext() {
    if (_cancelled || _nextIndex >= _addresses.length) return;
    final attempt = _MediaSocketAttempt<T>(
      _startConnect,
      _close,
      _addresses[_nextIndex],
      _port,
    );
    _nextIndex += 1;
    _activeAttempts += 1;
    _attempts.add(attempt);
    attempt.socket.then(
      (socket) => _accept(attempt, socket),
      onError: _reject,
    );
    _scheduleFallback();
  }

  void _scheduleFallback() {
    _timer?.cancel();
    if (_nextIndex >= _addresses.length ||
        _activeAttempts >= _maximumActiveAttempts) {
      return;
    }
    _timer = Timer(_fallbackDelay, _startNext);
  }

  void _accept(_MediaSocketAttempt<T> winner, T socket) {
    if (_result.isCompleted || _cancelled) {
      _close(socket);
      return;
    }
    _timer?.cancel();
    for (final attempt in _attempts) {
      if (!identical(attempt, winner)) attempt.cancel();
    }
    _result.complete(socket);
  }

  void _reject(Object error, StackTrace stackTrace) {
    _activeAttempts -= 1;
    _firstError ??= error;
    _firstStackTrace ??= stackTrace;
    if (_result.isCompleted || _cancelled) return;
    if (_nextIndex < _addresses.length) {
      _timer?.cancel();
      _startNext();
      return;
    }
    if (_activeAttempts == 0) {
      _result.completeError(_firstError!, _firstStackTrace!);
    }
  }

  @override
  void cancel() {
    if (_cancelled) return;
    _cancelled = true;
    _timer?.cancel();
    for (final attempt in _attempts) {
      attempt.cancel();
    }
    if (!_result.isCompleted) {
      _result.completeError(_cancellationError('media address', _port));
    }
  }
}
