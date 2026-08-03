part of 'public_media_http_client.dart';

class RawSecureSocketAdapter extends Stream<Uint8List> implements Socket {
  RawSecureSocketAdapter(this._raw) {
    _controller = StreamController<Uint8List>(sync: true)
      ..onListen = _resumeReads
      ..onPause = _pauseReads
      ..onResume = _resumeReads
      ..onCancel = _cancelReads;
    _raw.readEventsEnabled = false;
    _raw.writeEventsEnabled = false;
    _events = _raw.listen(
      _onRawEvent,
      onError: _onRawError,
      onDone: _onRawDone,
    );
    _sink = IOSink(_RawSecureSocketConsumer(this));
  }

  final RawSecureSocket _raw;
  late final StreamController<Uint8List> _controller;
  late final StreamSubscription<RawSocketEvent> _events;
  late final IOSink _sink;
  Completer<void>? _writable;
  var _inputClosed = false;
  var _readsPaused = true;
  var _destroyed = false;

  void _onRawEvent(RawSocketEvent event) {
    if (event == RawSocketEvent.read) {
      _drainReads();
      return;
    }
    if (event == RawSocketEvent.write) {
      _signalWritable();
      return;
    }
    if (event == RawSocketEvent.readClosed) _closeInput();
  }

  void _drainReads() {
    while (!_inputClosed && !_readsPaused) {
      final data = _raw.read();
      if (data == null) return;
      _controller.add(data);
    }
  }

  void _signalWritable() {
    _raw.writeEventsEnabled = false;
    final writable = _writable;
    _writable = null;
    if (writable != null && !writable.isCompleted) writable.complete();
  }

  void _onRawError(Object error, [StackTrace? stackTrace]) {
    final trace = stackTrace ?? StackTrace.current;
    if (!_inputClosed) _controller.addError(error, trace);
    _failWritable(error, trace);
  }

  void _onRawDone() {
    _closeInput();
    _failWritable(const SocketException('Secure socket closed.'));
  }

  void _closeInput() {
    if (_inputClosed) return;
    _inputClosed = true;
    _controller.close();
  }

  void _failWritable(Object error, [StackTrace? stackTrace]) {
    final writable = _writable;
    _writable = null;
    if (writable != null && !writable.isCompleted) {
      writable.completeError(error, stackTrace);
    }
  }

  Future<void> _writeAll(List<int> bytes) async {
    var offset = 0;
    while (offset < bytes.length) {
      if (_destroyed) throw const SocketException('Secure socket closed.');
      final written = _raw.write(bytes, offset);
      offset += written;
      if (written == 0) await _waitUntilWritable();
    }
  }

  Future<void> _waitUntilWritable() {
    final pending = _writable;
    if (pending != null) return pending.future;
    final writable = Completer<void>();
    _writable = writable;
    _raw.writeEventsEnabled = true;
    return writable.future;
  }

  Future<void> _closeOutput() async {
    if (_destroyed) return;
    _raw.shutdown(SocketDirection.send);
  }

  void _resumeReads() {
    _readsPaused = false;
    _raw.readEventsEnabled = true;
  }

  void _pauseReads() {
    _readsPaused = true;
    _raw.readEventsEnabled = false;
  }

  void _cancelReads() {
    _readsPaused = true;
    _raw.shutdown(SocketDirection.receive);
  }

  @override
  StreamSubscription<Uint8List> listen(
    void Function(Uint8List event)? onData, {
    Function? onError,
    void Function()? onDone,
    bool? cancelOnError,
  }) {
    return _controller.stream.listen(
      onData,
      onError: onError,
      onDone: onDone,
      cancelOnError: cancelOnError,
    );
  }

  @override
  void destroy() {
    if (_destroyed) return;
    _destroyed = true;
    _failWritable(const SocketException('Secure socket closed.'));
    _events.cancel().ignore();
    _raw.close().ignore();
    _closeInput();
  }

  @override
  void add(List<int> data) => _sink.add(data);
  @override
  void addError(Object error, [StackTrace? stackTrace]) =>
      throw UnsupportedError('Cannot send errors on sockets.');

  @override
  Future<void> addStream(Stream<List<int>> stream) => _sink.addStream(stream);
  @override
  Future<void> close() => _sink.close();
  @override
  Future<void> flush() => _sink.flush();
  @override
  Future<void> get done => _sink.done;

  @override
  Encoding get encoding => _sink.encoding;

  @override
  set encoding(Encoding value) => _sink.encoding = value;

  @override
  void write(Object? object) => _sink.write(object);

  @override
  void writeAll(Iterable<Object?> objects, [String separator = '']) =>
      _sink.writeAll(objects, separator);

  @override
  void writeCharCode(int charCode) => _sink.writeCharCode(charCode);

  @override
  void writeln([Object? object = '']) => _sink.writeln(object);

  @override
  InternetAddress get address => _raw.address;

  @override
  Uint8List getRawOption(RawSocketOption option) => _raw.getRawOption(option);

  @override
  int get port => _raw.port;

  @override
  InternetAddress get remoteAddress => _raw.remoteAddress;

  @override
  int get remotePort => _raw.remotePort;

  @override
  bool setOption(SocketOption option, bool enabled) =>
      _raw.setOption(option, enabled);

  @override
  void setRawOption(RawSocketOption option) => _raw.setRawOption(option);
}
