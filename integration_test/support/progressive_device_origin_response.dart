part of 'progressive_device_origin.dart';

extension _ProgressiveDeviceOriginResponse on ProgressiveDeviceOrigin {
  Future<bool> _write(
    HttpResponse response,
    ({int start, int end})? range,
    ProgressiveOriginRequest request,
  ) async {
    _concurrency.started(request.path, range);
    try {
      await _writeResponse(response, range, request);
      request._finish(
        ProgressiveOriginRequestOutcome.completed,
        _clock.elapsed,
      );
      return true;
    } on Object {
      request._finish(
        ProgressiveOriginRequestOutcome.clientCanceled,
        _clock.elapsed,
      );
      return false;
    } finally {
      _concurrency.finished(request.path, range);
    }
  }

  Future<void> _writeResponse(
    HttpResponse response,
    ({int start, int end})? range,
    ProgressiveOriginRequest request,
  ) async {
    final bytes = ProgressiveMp4Fixture.bytes;
    final span = range ?? (start: 0, end: bytes.length);
    _configure(response, range, span, bytes.length);
    await _streamChunks(response, bytes, span, request);
    await _closeResponse(response);
  }

  Future<void> _streamChunks(
    HttpResponse response,
    Uint8List bytes,
    ({int start, int end}) span,
    ProgressiveOriginRequest request,
  ) async {
    var firstChunk = true;
    for (var offset = span.start; offset < span.end;) {
      final end = (offset + _responseChunkBytes).clamp(offset, span.end);
      final hasMore = end < span.end;
      final event = await _sendChunk(response, (
        bytes: bytes,
        request: request,
        start: offset,
        end: end,
      ));
      _bandwidthTrigger?._afterChunk(request, hasMore, event);
      if (firstChunk) {
        await _firstChunkRendezvous?._afterFirstChunk(request);
        firstChunk = false;
      }
      await _chunkGate?._afterChunk(
        request,
        _requestSequences[request]!,
        hasMore,
      );
      offset = end;
      await _pacing.afterChunk(hasMore);
    }
  }

  void _configure(
    HttpResponse response,
    ({int start, int end})? range,
    ({int start, int end}) span,
    int total,
  ) {
    response.statusCode = range == null
        ? HttpStatus.ok
        : HttpStatus.partialContent;
    response.headers.contentType = ContentType('video', 'mp4');
    response.bufferOutput = false;
    response.headers.contentLength = span.end - span.start;
    response.headers.set(HttpHeaders.acceptRangesHeader, 'bytes');
    if (_validator == ProgressiveOriginValidator.stableStrong) {
      response.headers.set(HttpHeaders.etagHeader, '"warp-fixture-v1"');
    }
    if (range != null) _setContentRange(response, span, total);
  }
}
