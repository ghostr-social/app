import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import '../support/allow_all_media_url_policy.dart';

void main() {
  test('stops streaming before an in-flight file exceeds its allowance',
      () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-http-');
    addTearDown(() => directory.delete(recursive: true));
    final destination = File('${directory.path}/video.partial');
    final client = MockClient(
      (_) async => http.Response.bytes([1, 2, 3, 4], 200),
    );
    final downloader = HttpVideoFileDownloader(
      client,
      const AllowAllMediaUrlPolicy(),
    );

    final future = downloader.download(
      Uri.parse('https://media.test/video.mp4'),
      destination.path,
      maxBytes: 3,
    );

    await expectLater(future, throwsA(isA<AppFailure>()));
    expect(await destination.length(), lessThanOrEqualTo(3));
  });
}
