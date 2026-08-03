import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('refuses a media redirect to a private address', () async {
    final attempted = <Uri>[];
    final client = MockClient((request) async {
      attempted.add(request.url);
      return http.Response(
        '',
        302,
        headers: {'location': 'http://127.0.0.1/private.mp4'},
      );
    });
    final policy = PublicMediaAddressResolver(
      lookup: (host) async => [
        host == 'media.test'
            ? InternetAddress('93.184.216.34')
            : InternetAddress.loopbackIPv4,
      ],
    );
    final downloader = HttpVideoFileDownloader(client, policy);

    final future = downloader.download(
      Uri.parse('https://media.test/video.mp4'),
      '/unwritten/video.partial',
      maxBytes: 10,
    );

    await expectLater(
      future,
      throwsA(isA<AppFailure>().having(
        (failure) => failure.message,
        'message',
        contains('public internet address'),
      )),
    );
    expect(attempted, [Uri.parse('https://media.test/video.mp4')]);
  });
}
