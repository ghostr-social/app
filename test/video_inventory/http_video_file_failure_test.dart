import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import '../support/allow_all_media_url_policy.dart';

void main() {
  test('translates a failed HTTP response into an app-safe failure', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-http-');
    addTearDown(() => directory.delete(recursive: true));
    final destination = File('${directory.path}/video.partial');
    final client = MockClient((_) async => http.Response('', 503));
    final downloader = HttpVideoFileDownloader(
      client,
      const AllowAllMediaUrlPolicy(),
    );

    final future = downloader.download(
      Uri.parse('https://media.test/video.mp4'),
      destination.path,
      maxBytes: 10,
    );

    await expectLater(
      future,
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        contains('503'),
      )),
    );
  });
}
