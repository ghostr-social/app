import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/media/video_download_timeouts.dart';
import 'package:http/io_client.dart';

import '../support/allow_all_media_url_policy.dart';

void main() {
  test('cancels a rejected never-ending IO response body', () async {
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    final serving = server.listen((request) => unawaited(_serve(request)));
    addTearDown(() async {
      await serving.cancel();
      await server.close(force: true);
    });
    final directory = await Directory.systemTemp.createTemp('ghostr-http-');
    addTearDown(() => directory.delete(recursive: true));
    final ioClient = HttpClient()..maxConnectionsPerHost = 1;
    final client = IOClient(ioClient);
    addTearDown(client.close);
    final downloader = HttpVideoFileDownloader(
      client,
      const AllowAllMediaUrlPolicy(),
      timeouts: const VideoDownloadTimeouts(
        headers: Duration(seconds: 1),
        idle: Duration(seconds: 1),
        total: Duration(seconds: 2),
      ),
    );

    final future = downloader.download(
      Uri.parse('http://${server.address.address}:${server.port}/rejected'),
      '${directory.path}/video.partial',
      maxBytes: 10,
    );

    await expectLater(
      future.timeout(const Duration(milliseconds: 500)),
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        contains('503'),
      )),
    );
    await downloader
        .download(
          Uri.parse('http://${server.address.address}:${server.port}/ok'),
          '${directory.path}/next.partial',
          maxBytes: 10,
        )
        .timeout(const Duration(milliseconds: 500));
  });
}

Future<void> _serve(HttpRequest request) async {
  if (request.uri.path == '/ok') {
    request.response.add(const <int>[1]);
    await request.response.close();
    return;
  }
  request.response
    ..statusCode = HttpStatus.serviceUnavailable
    ..bufferOutput = false;
  try {
    while (true) {
      request.response.add(const <int>[1]);
      await request.response.flush();
      await Future<void>.delayed(const Duration(milliseconds: 5));
    }
  } on Object {
    return;
  }
}
