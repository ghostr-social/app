part of 'public_media_http_client.dart';

const _fallbackDelay = Duration(milliseconds: 250);
const _maximumActiveAttempts = 2;

class _PublicMediaConnectionFactory {
  _PublicMediaConnectionFactory(
    this._resolver,
    PublicMediaHttpClientConfig config,
  )   : _startConnect = config.startConnect ?? Socket.startConnect,
        _startRawConnect =
            config.startRawConnect ?? MediaRawSocketTask.startConnect,
        _securityContext = config.securityContext;

  final PublicMediaAddressResolver _resolver;
  final MediaSocketStarter _startConnect;
  final MediaRawSocketStarter _startRawConnect;
  final SecurityContext? _securityContext;

  Future<ConnectionTask<Socket>> connect(
    Uri source,
    String? proxyHost,
    int? proxyPort,
  ) {
    final connection = _PublicMediaConnection(this, source);
    return Future.value(connection.task);
  }
}

class _PublicMediaConnection {
  _PublicMediaConnection(this._factory, this._source) {
    task = ConnectionTask.fromSocket(_result.future, cancel);
    _open().then(_complete, onError: _fail);
  }

  final _PublicMediaConnectionFactory _factory;
  final Uri _source;
  final Completer<Socket> _result = Completer<Socket>();
  late final ConnectionTask<Socket> task;
  _CancelableConnection? _race;
  var _cancelled = false;

  Future<Socket> _open() async {
    final addresses = await _resolveAddresses();
    _requireActive();
    if (_source.scheme == 'https') return _openSecure(addresses);
    return _openPlain(addresses);
  }

  Future<List<InternetAddress>> _resolveAddresses() async {
    try {
      return await _factory._resolver.resolveAll(_source);
    } on AppFailure catch (error) {
      throw SocketException(error.message);
    }
  }

  Future<Socket> _openPlain(List<InternetAddress> addresses) async {
    final race = _MediaConnectionRace<Socket>(
      _factory._startConnect,
      _destroySocket,
      _port,
      _fallbackDelay,
    );
    _race = race;
    final socket = await race.connect(addresses);
    _requireActive();
    return socket;
  }

  Future<Socket> _openSecure(List<InternetAddress> addresses) async {
    final starter = _SecureMediaSocketStarter(
      _factory._startRawConnect,
      _source.host,
      _factory._securityContext,
    );
    final race = _MediaConnectionRace<Socket>(
      starter.start,
      _destroySocket,
      _port,
      _fallbackDelay,
    );
    _race = race;
    final socket = await race.connect(addresses);
    _requireActive();
    return socket;
  }

  int get _port {
    if (_source.hasPort) return _source.port;
    return _source.scheme == 'https' ? 443 : 80;
  }

  void _requireActive() {
    if (_cancelled) throw _cancellationError(_source.host, _port);
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
    _race?.cancel();
    _fail(_cancellationError(_source.host, _port), StackTrace.current);
  }
}

void _destroySocket(Socket socket) => socket.destroy();

SocketException _cancellationError(String host, int port) {
  return SocketException('Connection cancelled, host: $host, port: $port');
}
