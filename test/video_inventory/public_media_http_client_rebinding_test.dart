import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

void main() {
  test('rejects a private DNS result immediately before connecting', () async {
    var lookups = 0;
    var connectionStarted = false;
    final resolver = PublicMediaAddressResolver(
      lookup: (_) async {
        lookups += 1;
        return [
          InternetAddress(lookups == 1 ? '8.8.8.8' : '127.0.0.1'),
        ];
      },
    );
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startRawConnect: (address, port) async {
          connectionStarted = true;
          throw StateError('A rejected address must never be connected.');
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

    expect(lookups, 2);
    expect(connectionStarted, isFalse);
  });
}
