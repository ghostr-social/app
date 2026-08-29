part of 'progressive_device_origin.dart';

const _progressiveResponseIoDeadline = Duration(seconds: 1);
const _progressiveResponseIoTimeout = Duration(milliseconds: 1250);

typedef _ProgressiveResponseChunk = ({
  Uint8List bytes,
  ProgressiveOriginRequest request,
  int start,
  int end,
});

extension _ProgressiveDeviceOriginSend on ProgressiveDeviceOrigin {
  Future<ProgressiveOriginChunkEvent?> _sendChunk(
    HttpResponse response,
    _ProgressiveResponseChunk chunk,
  ) async {
    await _preBodyGate?._beforeFirstBody(chunk.request);
    final permit = await _pacing.acquire(chunk.end - chunk.start);
    ProgressiveOriginChunkEvent? event;
    try {
      response.add(Uint8List.sublistView(chunk.bytes, chunk.start, chunk.end));
      if (permit != null) event = _recordPacedChunk(permit, chunk);
    } finally {
      permit?.release();
    }
    await _flushResponse(response);
    _recordBytes(chunk.request, chunk.end - chunk.start);
    if (event != null) _pacing.confirm(event);
    return event;
  }

  ProgressiveOriginChunkEvent _recordPacedChunk(
    _ProgressiveOriginChunkPermit permit,
    _ProgressiveResponseChunk chunk,
  ) {
    return _pacing.record(permit, (
      requestSequence: _requestSequences[chunk.request]!,
      path: chunk.request.path,
      start: chunk.start,
      end: chunk.end,
    ));
  }

  Future<void> _flushResponse(HttpResponse response) async {
    var expired = false;
    response.deadline = _progressiveResponseIoDeadline;
    try {
      await response.flush().timeout(
        _progressiveResponseIoTimeout,
        onTimeout: () {
          expired = true;
          return _expireResponse(response);
        },
      );
    } finally {
      if (!expired) response.deadline = null;
    }
  }

  Future<void> _closeResponse(HttpResponse response) async {
    var expired = false;
    response.deadline = _progressiveResponseIoDeadline;
    try {
      await response.close().timeout(
        _progressiveResponseIoTimeout,
        onTimeout: () {
          expired = true;
          return _expireResponse(response);
        },
      );
    } finally {
      if (!expired) response.deadline = null;
    }
  }
}

Never _expireResponse(HttpResponse response) {
  response.deadline = Duration.zero;
  throw TimeoutException('Progressive origin response I/O timed out.');
}
