part of 'progressive_device_origin.dart';

extension ProgressiveDeviceOriginLifecycle on ProgressiveDeviceOrigin {
  void _dispatch(HttpRequest request) => unawaited(_serve(request));

  Future<void> _serve(HttpRequest request) async {
    try {
      await _handle(request);
    } on Object catch (error, stackTrace) {
      log(
        'Progressive device origin request failed.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  Future<void> _handle(HttpRequest request) async {
    if (await _handleHls(request)) return;
    final range = _requestedRange(request, ProgressiveMp4Fixture.bytes.length);
    final entry = ProgressiveOriginRequest(
      request.method,
      request.uri.path,
      range,
      startedAt: _clock.elapsed,
    );
    requests.add(entry);
    _requestSequences[entry] = ++_nextRequestSequence;
    if (_availability == ProgressiveOriginAvailability.unavailable) {
      return _rejectUnavailable(request.response, entry);
    }
    if (request.method == 'HEAD') {
      return _holdHead(request.response, entry);
    }
    final completed = await _write(request.response, range, entry);
    if (completed) _completed.add(entry);
  }

  Future<void> _holdHead(
    HttpResponse response,
    ProgressiveOriginRequest entry,
  ) async {
    final socket = await response.detachSocket(writeHeaders: false);
    entry._blockHead();
    _heldHeads.add(socket);
    void closed() {
      if (!_heldHeads.remove(socket)) return;
      _peerClosed(entry);
      entry._finish(
        ProgressiveOriginRequestOutcome.clientCanceled,
        _clock.elapsed,
      );
      socket.destroy();
    }

    socket.listen((_) {}, onDone: closed, onError: (Object _) => closed());
  }

  Future<void> _rejectUnavailable(
    HttpResponse response,
    ProgressiveOriginRequest request,
  ) async {
    response.statusCode = HttpStatus.serviceUnavailable;
    response.headers.contentLength = 0;
    await response.close();
    request._finish(ProgressiveOriginRequestOutcome.completed, _clock.elapsed);
  }

  void _recordBytes(ProgressiveOriginRequest request, int count) {
    request._recordBytes(count, _clock.elapsed);
    _servedBytes.update(
      request.path,
      (total) => total + count,
      ifAbsent: () => count,
    );
  }

  Future<void> close() async {
    _firstChunkRendezvous?.release();
    _bandwidthTrigger?.cancel();
    _chunkGate?.release();
    _preBodyGate?.release();
    for (final socket in _heldHeads.toList()) {
      socket.destroy();
    }
    _heldHeads.clear();
    await _subscription.cancel();
    await _server.close(force: true);
  }
}
