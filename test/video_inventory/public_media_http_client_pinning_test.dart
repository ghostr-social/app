import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

void main() {
  test('connects to the validated numeric address instead of the hostname',
      () async {
    final validated = InternetAddress('8.8.8.8');
    InternetAddress? connected;
    final resolver = PublicMediaAddressResolver(
      lookup: (_) async => [validated],
    );
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startRawConnect: (address, port) async {
          connected = address;
          throw const SocketException('Expected test connection failure.');
        },
      ),
    );
    addTearDown(client.close);

    final downloader = HttpVideoFileDownloader(client, resolver);
    await expectLater(
      downloader.download(
        Uri.parse('https://media.example/video.mp4'),
        'unused.mp4',
        maxBytes: 1024,
      ),
      throwsA(isA<AppFailure>()),
    );

    expect(connected, validated);
  });
}
