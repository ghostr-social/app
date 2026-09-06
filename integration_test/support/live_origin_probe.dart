import 'dart:io';

import 'live_video_log.dart';

// Run only after playback measurements, so this request cannot prime them.
Future<void> liveOriginProbe(LiveVideoLog log, Uri url) async {
  final client = HttpClient()..connectionTimeout = const Duration(seconds: 10);
  final clock = Stopwatch()..start();
  try {
    final request = await client.getUrl(url);
    request.headers.set(HttpHeaders.rangeHeader, 'bytes=0-65535');
    final response = await request.close().timeout(const Duration(seconds: 15));
    log.add('origin_headers', {
      'url': '$url',
      'headersMs': clock.elapsedMilliseconds,
      'status': response.statusCode,
      'contentType': response.headers.value('content-type'),
      'contentLength': response.contentLength,
      'contentRange': response.headers.value('content-range'),
      'acceptRanges': response.headers.value('accept-ranges'),
      'redirects': response.redirects.map((r) => '${r.location}').toList(),
    });
    var bytes = 0;
    await for (final chunk in response.timeout(const Duration(seconds: 10))) {
      bytes += chunk.length;
      if (bytes >= 65536) break;
    }
    log.add('origin_body', {
      'url': '$url',
      'bytes': bytes,
      'durationMs': clock.elapsedMilliseconds,
    });
  } on Object catch (error) {
    log.add('origin_error', {
      'url': '$url',
      'durationMs': clock.elapsedMilliseconds,
      'error': '$error',
    });
  } finally {
    client.close(force: true);
  }
}
