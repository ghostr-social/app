part of 'device_video_server.dart';

extension _DeviceVideoServerResponses on DeviceVideoServer {
  Uint8List? _asset(String name) {
    if (name == 'index.m3u8') {
      return Uint8List.fromList(utf8.encode(DeterministicHlsFixture.playlist));
    }
    return DeterministicHlsFixture.assets[name];
  }

  Future<void> _write(
    HttpResponse response,
    String asset,
    Uint8List bytes,
  ) async {
    response.headers.contentType = _contentType(asset);
    response.headers.set(HttpHeaders.cacheControlHeader, 'no-store');
    response.contentLength = bytes.length;
    try {
      await _writeBody(response, asset, bytes);
      await response.close();
      if (_isSuccessfulManifestRetry(asset)) {
        successfulManifestResponses += 1;
      }
    } on Object {
      cancellationWasteBytes += bytes.length;
    }
  }

  bool _isSuccessfulManifestRetry(String asset) {
    return scenario == DeviceVideoScenario.manifestRetry &&
        asset == 'index.m3u8' &&
        manifestFailures > 0;
  }

  Future<void> _writeBody(
    HttpResponse response,
    String asset,
    Uint8List bytes,
  ) async {
    if (!_throttlesBandwidth(scenario, asset)) {
      response.add(bytes);
      return;
    }
    impairedResponses += 1;
    final chunkSize = (bytes.length + 3) ~/ 4;
    for (var offset = 0; offset < bytes.length; offset += chunkSize) {
      final end = (offset + chunkSize).clamp(0, bytes.length);
      response.add(bytes.sublist(offset, end));
      if (end < bytes.length) {
        await Future<void>.delayed(const Duration(milliseconds: 250));
      }
    }
  }
}

ContentType _contentType(String asset) => asset.endsWith('.m3u8')
    ? ContentType('application', 'vnd.apple.mpegurl')
    : ContentType('video', asset.endsWith('.mp4') ? 'mp4' : 'iso.segment');

String _sessionId(String value) {
  final seed = utf8.encode(value).map(_hexByte).join();
  return List<String>.generate(64, (index) => seed[index % seed.length]).join();
}

String _hexByte(int value) => value.toRadixString(16).padLeft(2, '0');

Future<void> _notFound(HttpResponse response) async {
  response.statusCode = HttpStatus.notFound;
  await response.close();
}

Future<void> _serviceUnavailable(HttpResponse response) async {
  response.statusCode = HttpStatus.serviceUnavailable;
  await response.close();
}
