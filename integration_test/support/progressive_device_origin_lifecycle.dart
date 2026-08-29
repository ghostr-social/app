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
    final range = _requestedRange(request, ProgressiveMp4Fixture.bytes.length);
    final entry = ProgressiveOriginRequest(
      request.method,
      request.uri.path,
      range,
      startedAt: _clock.elapsed,
    );
    requests.add(entry);
    _requestSequences[entry] = ++_nextRequestSequence;
    if (request.method == 'HEAD') {
      entry._blockHead();
      _heldHeads.add(request.response);
      return;
    }
    final completed = await _write(request.response, range, entry);
    if (completed) _completed.add(entry);
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
    for (final response in _heldHeads) {
      await response.close();
    }
    await _subscription.cancel();
    await _server.close(force: true);
  }
}
