part of 'progressive_device_origin.dart';

extension _ProgressiveDeviceOriginDetachedResponse on ProgressiveDeviceOrigin {
  Future<void> _writeDetachedResponse(
    HttpResponse response,
    Uint8List bytes,
    ({int start, int end}) span,
    ProgressiveOriginRequest request,
  ) async {
    final socket = await response.detachSocket();
    final peer = _observePeer(socket, request);
    try {
      await _flushSocket(socket);
      await _streamSocketChunks(socket, bytes, span, request);
      await _closeSocket(socket);
    } on Object {
      socket.destroy();
      rethrow;
    } finally {
      await peer.cancel();
    }
  }

  StreamSubscription<Uint8List> _observePeer(
    Socket socket,
    ProgressiveOriginRequest request,
  ) => socket.listen(
    (_) {},
    onError: (_, _) => _preBodyGate?._observePeerClosed(request),
    onDone: () => _preBodyGate?._observePeerClosed(request),
  );

  Future<void> _streamSocketChunks(
    Socket socket,
    Uint8List bytes,
    ({int start, int end}) span,
    ProgressiveOriginRequest request,
  ) async {
    var first = true;
    for (var offset = span.start; offset < span.end;) {
      final end = (offset + _responseChunkBytes).clamp(offset, span.end);
      final event = await _sendSocketChunk(socket, bytes, request, (
        offset,
        end,
      ));
      await _afterSocketChunk(request, event, end < span.end, first);
      first = false;
      offset = end;
      await _pacing.afterChunk(offset < span.end);
    }
  }

  Future<ProgressiveOriginChunkEvent?> _sendSocketChunk(
    Socket socket,
    Uint8List bytes,
    ProgressiveOriginRequest request,
    (int, int) span,
  ) async {
    await _preBodyGate?._beforeFirstBody(request);
    final permit = await _pacing.acquire(span.$2 - span.$1);
    ProgressiveOriginChunkEvent? event;
    try {
      socket.add(Uint8List.sublistView(bytes, span.$1, span.$2));
      if (permit != null) {
        event = _recordPacedChunk(permit, (
          bytes: bytes,
          request: request,
          start: span.$1,
          end: span.$2,
        ));
      }
    } finally {
      permit?.release();
    }
    await _flushSocket(socket);
    _recordBytes(request, span.$2 - span.$1);
    if (event != null) _pacing.confirm(event);
    return event;
  }

  Future<void> _afterSocketChunk(
    ProgressiveOriginRequest request,
    ProgressiveOriginChunkEvent? event,
    bool hasMore,
    bool first,
  ) async {
    _bandwidthTrigger?._afterChunk(request, hasMore, event);
    if (first) await _firstChunkRendezvous?._afterFirstChunk(request);
    await _chunkGate?._afterChunk(
      request,
      _requestSequences[request]!,
      hasMore,
    );
  }

  Future<void> _flushSocket(Socket socket) => socket.flush().timeout(
    _progressiveResponseIoTimeout,
    onTimeout: () => _expireSocket(socket),
  );

  Future<void> _closeSocket(Socket socket) => socket.close().timeout(
    _progressiveResponseIoTimeout,
    onTimeout: () => _expireSocket(socket),
  );
}

Never _expireSocket(Socket socket) {
  socket.destroy();
  throw TimeoutException('Progressive origin socket I/O timed out.');
}
