part of 'progressive_device_origin.dart';

extension _ProgressiveDeviceOriginResponse on ProgressiveDeviceOrigin {
  Future<bool> _write(
    HttpResponse response,
    ({int start, int end})? range,
    ProgressiveOriginRequest request,
  ) async {
    _concurrency.started(request.path, range);
    try {
      final bytes = ProgressiveMp4Fixture.bytes;
      final span = range ?? (start: 0, end: bytes.length);
      _configure(response, range, span, bytes.length);
      for (var offset = span.start; offset < span.end;) {
        final end = (offset + _responseChunkBytes).clamp(offset, span.end);
        response.add(Uint8List.sublistView(bytes, offset, end));
        await response.flush();
        _recordBytes(request, end - offset);
        offset = end;
        if (offset < span.end && _responseChunkDelay > Duration.zero) {
          await Future<void>.delayed(_responseChunkDelay);
        }
      }
      await response.close();
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
    response.headers.contentLength = span.end - span.start;
    response.headers.set(HttpHeaders.acceptRangesHeader, 'bytes');
    if (_validator == ProgressiveOriginValidator.stableStrong) {
      response.headers.set(HttpHeaders.etagHeader, '"warp-fixture-v1"');
    }
    if (range != null) _setContentRange(response, span, total);
  }
}

({int start, int end})? _requestedRange(HttpRequest request, int total) {
  final raw = request.headers.value(HttpHeaders.rangeHeader);
  if (raw == null || !raw.startsWith('bytes=')) return null;
  final bounds = raw.substring(6).split('-');
  final start = int.tryParse(bounds.first) ?? 0;
  final inclusiveEnd = int.tryParse(bounds.last) ?? total - 1;
  return (
    start: start.clamp(0, total - 1),
    end: (inclusiveEnd + 1).clamp(1, total),
  );
}

void _setContentRange(
  HttpResponse response,
  ({int start, int end}) range,
  int total,
) {
  response.headers.set(
    HttpHeaders.contentRangeHeader,
    'bytes ${range.start}-${range.end - 1}/$total',
  );
}
