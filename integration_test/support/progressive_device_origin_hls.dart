part of 'progressive_device_origin.dart';

extension ProgressiveDeviceOriginHls on ProgressiveDeviceOrigin {
  Uri hlsUrlFor(String id) => Uri.parse(
    'http://${_server.address.address}:${_server.port}/hls/$id/index.m3u8',
  );

  int hlsRequestsFor(String asset) =>
      requests.where((request) => request.path.endsWith('/$asset')).length;

  Future<bool> _handleHls(HttpRequest request) async {
    final parts = request.uri.pathSegments;
    if (parts.length != 3 || parts.first != 'hls') return false;
    final bytes = _hlsAsset(parts.last);
    if (bytes == null) {
      request.response.statusCode = HttpStatus.notFound;
      await request.response.close();
      return true;
    }
    final entry = ProgressiveOriginRequest(
      request.method,
      request.uri.path,
      null,
      startedAt: _clock.elapsed,
    );
    requests.add(entry);
    _requestSequences[entry] = ++_nextRequestSequence;
    await _writeHls(request, entry, bytes);
    return true;
  }

  Future<void> _writeHls(
    HttpRequest request,
    ProgressiveOriginRequest entry,
    Uint8List bytes,
  ) async {
    final response = request.response;
    response.headers.contentType = _hlsContentType(request.uri.path);
    response.headers.contentLength = bytes.length;
    response.headers.set(HttpHeaders.cacheControlHeader, 'no-store');
    if (request.method != 'HEAD') response.add(bytes);
    await response.close();
    _recordBytes(entry, request.method == 'HEAD' ? 0 : bytes.length);
    entry._finish(ProgressiveOriginRequestOutcome.completed, _clock.elapsed);
    _completed.add(entry);
  }
}

Uint8List? _hlsAsset(String asset) {
  if (asset == 'index.m3u8') {
    return Uint8List.fromList(utf8.encode(DeterministicHlsFixture.playlist));
  }
  return DeterministicHlsFixture.assets[asset];
}

ContentType _hlsContentType(String path) => path.endsWith('.m3u8')
    ? ContentType('application', 'vnd.apple.mpegurl')
    : ContentType('video', path.endsWith('.mp4') ? 'mp4' : 'iso.segment');
