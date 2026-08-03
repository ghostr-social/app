import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

void main() {
  test('refuses media hosts that resolve to a private address', () async {
    var requestWasSent = false;
    final client = MockClient((_) async {
      requestWasSent = true;
      return http.Response('', 200);
    });
    final policy = PublicMediaAddressResolver(
      lookup: (_) async => [InternetAddress.loopbackIPv4],
    );
    final downloader = HttpVideoFileDownloader(client, policy);

    final future = downloader.download(
      Uri.parse('https://media.invalid/video.mp4'),
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
    expect(requestWasSent, isFalse);
  });
}
