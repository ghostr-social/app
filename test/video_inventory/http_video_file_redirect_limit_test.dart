import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('fails after five media redirects', () async {
    var requests = 0;
    final client = MockClient((_) async {
      requests += 1;
      return http.Response('', 302, headers: {'location': '/loop.mp4'});
    });
    final policy = PublicMediaAddressResolver(
      lookup: (_) async => [InternetAddress('93.184.216.34')],
    );
    final downloader = HttpVideoFileDownloader(client, policy);

    final future = downloader.download(
      Uri.parse('https://media.test/source.mp4'),
      '/unwritten/video.partial',
      maxBytes: 10,
    );

    await expectLater(
      future,
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        contains('redirect limit'),
      )),
    );
    expect(requests, 6);
  });
}
