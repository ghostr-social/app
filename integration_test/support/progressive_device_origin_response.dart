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
    final plan = _responsePlan(request.path, range, bytes.length);
    _configure(response, plan);
    if (_preBodyGate?._matches(request) ?? false) {
      await _writeDetachedResponse(response, bytes, plan.span, request);
      return;
    }
    await _streamChunks(response, bytes, plan.span, request);
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

  void _configure(HttpResponse response, _ProgressiveOriginResponsePlan plan) {
    response.statusCode = plan.statusCode;
    response.headers.contentType = ContentType('video', 'mp4');
    response.bufferOutput = false;
    response.headers.contentLength = plan.span.end - plan.span.start;
    response.headers.set(HttpHeaders.acceptRangesHeader, 'bytes');
    if (_validator == ProgressiveOriginValidator.stableStrong) {
      response.headers.set(HttpHeaders.etagHeader, '"warp-fixture-v1"');
    }
    if (plan.contentRange case final contentRange?) {
      response.headers.set(HttpHeaders.contentRangeHeader, contentRange);
    }
  }
}
