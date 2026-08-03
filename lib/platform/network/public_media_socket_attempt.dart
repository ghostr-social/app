part of 'public_media_http_client.dart';

typedef _MediaConnectionStarter<T> = Future<ConnectionTask<T>> Function(
  InternetAddress address,
  int port,
);

typedef _MediaConnectionCloser<T> = void Function(T socket);

class _MediaSocketAttempt<T> {
  _MediaSocketAttempt(
    this._startConnect,
    this._close,
    this._address,
    this._port,
  ) {
    socket = _open();
  }

  final _MediaConnectionStarter<T> _startConnect;
  final _MediaConnectionCloser<T> _close;
  final InternetAddress _address;
  final int _port;
  late final Future<T> socket;
  ConnectionTask<T>? _task;
  T? _socket;
  var _cancelled = false;

  Future<T> _open() async {
    final task = await _startConnect(_address, _port);
    _task = task;
    if (_cancelled) {
      task.socket.ignore();
      task.cancel();
      throw _cancellationError(_address.address, _port);
    }
    final connected = await task.socket;
    _socket = connected;
    if (!_cancelled) return connected;
    _close(connected);
    throw _cancellationError(_address.address, _port);
  }

  void cancel() {
    if (_cancelled) return;
    _cancelled = true;
    _task?.cancel();
    final connected = _socket;
    if (connected != null) _close(connected);
  }
}
