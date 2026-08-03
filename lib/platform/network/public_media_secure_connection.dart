part of 'public_media_http_client.dart';

class _SecureMediaSocketStarter {
  const _SecureMediaSocketStarter(
    this._startRawConnect,
    this._host,
    this._securityContext,
  );

  final MediaRawSocketStarter _startRawConnect;
  final String _host;
  final SecurityContext? _securityContext;

  Future<ConnectionTask<Socket>> start(
    InternetAddress address,
    int port,
  ) {
    final target = _SecureMediaTarget(
      address,
      port,
      _host,
      _securityContext,
    );
    return Future.value(_SecureMediaConnection(_startRawConnect, target).task);
  }
}

class _SecureMediaTarget {
  const _SecureMediaTarget(
    this.address,
    this.port,
    this.host,
    this.securityContext,
  );

  final InternetAddress address;
  final int port;
  final String host;
  final SecurityContext? securityContext;
}

class _SecureMediaConnection {
  _SecureMediaConnection(this._startRawConnect, this._target) {
    task = ConnectionTask.fromSocket(_result.future, cancel);
    _open().then(_complete, onError: _fail);
  }

  final MediaRawSocketStarter _startRawConnect;
  final _SecureMediaTarget _target;
  final Completer<Socket> _result = Completer<Socket>();
  late final ConnectionTask<Socket> task;
  MediaRawSocketTask? _rawTask;
  RawSocket? _raw;
  Socket? _socket;
  var _cancelled = false;

  Future<Socket> _open() async {
    final task = await _startRawConnect(_target.address, _target.port);
    _rawTask = task;
    if (_cancelled) {
      task.socket.ignore();
      task.cancel();
      throw _cancelError;
    }
    final raw = await task.socket;
    _raw = raw;
    if (_cancelled) {
      raw.close().ignore();
      throw _cancelError;
    }
    final secure = await _upgrade(raw);
    final socket = RawSecureSocketAdapter(secure);
    _socket = socket;
    return socket;
  }

  Future<RawSecureSocket> _upgrade(RawSocket raw) async {
    try {
      return await RawSecureSocket.secure(
        raw,
        host: _target.host,
        context: _target.securityContext,
      );
    } on Object {
      raw.close().ignore();
      rethrow;
    }
  }

  SocketException get _cancelError {
    return _cancellationError(_target.address.address, _target.port);
  }

  void _complete(Socket socket) {
    if (_result.isCompleted) {
      socket.destroy();
      return;
    }
    _result.complete(socket);
  }

  void _fail(Object error, StackTrace stackTrace) {
    if (!_result.isCompleted) _result.completeError(error, stackTrace);
  }

  void cancel() {
    if (_cancelled) return;
    _cancelled = true;
    _rawTask?.cancel();
    _raw?.close().ignore();
    _socket?.destroy();
    _fail(_cancelError, StackTrace.current);
  }
}
