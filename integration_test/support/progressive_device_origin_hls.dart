part of 'progressive_device_origin.dart';

extension ProgressiveDeviceOriginHls on ProgressiveDeviceOrigin {
  Uri get encryptedHlsUrl => Uri.parse(
    'http://${_server.address.address}:${_server.port}$_encryptedHlsPath',
  );

  Uri hlsUrlFor(String id) => Uri.parse(
    'http://${_server.address.address}:${_server.port}/hls/$id/index.m3u8',
  );

  List<ProgressiveOriginRequest> get encryptedHlsRequests => requests
      .where((request) => request.path == _encryptedHlsPath)
      .toList(growable: false);

  int hlsRequestsFor(String asset) =>
      requests.where((request) => request.path.endsWith('/$asset')).length;

  Future<bool> _handleHls(HttpRequest request) async {
    if (request.uri.path == _encryptedHlsPath) {
      await _serveHls(request, utf8.encode(_encryptedVodManifest));
      return true;
    }
    final parts = request.uri.pathSegments;
    if (parts.length != 3 || parts.first != 'hls') return false;
    final bytes = ProgressiveDeviceHlsAssets.resolve(parts[1], parts.last);
    if (bytes == null) {
      request.response.statusCode = HttpStatus.notFound;
      await request.response.close();
      return true;
    }
    await _serveHls(request, bytes);
    return true;
  }

  Future<void> _serveHls(HttpRequest request, List<int> bytes) async {
    final entry = ProgressiveOriginRequest(
      request.method,
      request.uri.path,
      null,
      startedAt: _clock.elapsed,
    );
    requests.add(entry);
    _requestSequences[entry] = ++_nextRequestSequence;
    await _writeHls(request, entry, bytes);
  }

  Future<void> _writeHls(
    HttpRequest request,
    ProgressiveOriginRequest entry,
    List<int> bytes,
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

ContentType _hlsContentType(String path) => path.endsWith('.m3u8')
    ? ContentType('application', 'vnd.apple.mpegurl')
    : ContentType('video', path.endsWith('.mp4') ? 'mp4' : 'iso.segment');

const _encryptedHlsPath = '/encrypted/index.m3u8';

const _encryptedVodManifest = '''#EXTM3U
#EXT-X-VERSION:7
#EXT-X-TARGETDURATION:1
#EXT-X-KEY:METHOD=AES-128,URI="key.bin"
#EXT-X-MAP:URI="init.mp4"
#EXTINF:1.0,
index0.m4s
#EXT-X-ENDLIST
''';
