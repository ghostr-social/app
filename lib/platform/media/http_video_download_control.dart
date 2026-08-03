part of 'http_video_file_downloader.dart';

class _DownloadResponse {
  _DownloadResponse(this.response, this._control, this._idleTimeout);

  final http.StreamedResponse response;
  final _RequestControl _control;
  final Duration _idleTimeout;
  bool _consumed = false;

  Stream<List<int>> get stream {
    if (_consumed) {
      throw StateError('Video response body was already consumed.');
    }
    _consumed = true;
    return response.stream.timeout(_idleTimeout);
  }

  Future<void> discard() async {
    if (_consumed) return;
    _consumed = true;
    final subscription = response.stream.listen(
      null,
      onError: (Object _, StackTrace __) {},
    );
    try {
      _control.abort();
      await subscription.cancel();
    } finally {
      _control.close();
    }
  }

  Future<T> untilAborted<T>(Future<T> operation) {
    return _control.untilAbortedAndSettled(operation);
  }

  void close() => _control.close();
}

class _RequestControl {
  _RequestControl(Duration totalTimeout, [Future<void>? parent]) {
    _timer = Timer(totalTimeout, abort);
    parent?.then((_) => abort());
  }

  final Completer<void> _abort = Completer<void>();
  late final Timer _timer;

  Future<void> get aborted => _abort.future;

  void abort() {
    if (!_abort.isCompleted) _abort.complete();
  }

  void requireActive() {
    if (_abort.isCompleted) throw TimeoutException('Video download timed out.');
  }

  Future<T> untilAborted<T>(Future<T> operation) {
    return _race(operation, settleAfterAbort: false);
  }

  Future<T> untilAbortedAndSettled<T>(Future<T> operation) {
    return _race(operation, settleAfterAbort: true);
  }

  Future<T> _race<T>(
    Future<T> operation, {
    required bool settleAfterAbort,
  }) async {
    try {
      return await Future.any(<Future<T>>[operation, _abortedResult<T>()]);
    } on _RequestAborted {
      if (settleAfterAbort) await _settle(operation);
      throw TimeoutException('Video download timed out.');
    }
  }

  Future<T> _abortedResult<T>() async {
    await aborted;
    throw const _RequestAborted();
  }

  Future<void> _settle<T>(Future<T> operation) async {
    try {
      await operation;
    } on Object {
      return;
    }
  }

  void cancelTimer() => _timer.cancel();

  void close() {
    _timer.cancel();
    abort();
  }
}

class _RequestAborted implements Exception {
  const _RequestAborted();
}
