import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/media/video_download_timeouts.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

void main() {
  test('connection timeout covers the pre-connect DNS lookup', () async {
    final pendingLookup = Completer<List<InternetAddress>>();
    var lookups = 0;
    var connectionStarted = false;
    final resolver = PublicMediaAddressResolver(
      lookup: (_) {
        lookups += 1;
        if (lookups == 1) {
          return Future.value([InternetAddress('8.8.8.8')]);
        }
        return pendingLookup.future;
      },
    );
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startConnect: (address, port) async {
          connectionStarted = true;
          throw StateError('Connection must not start before DNS completes.');
        },
        connectionTimeout: const Duration(milliseconds: 100),
      ),
    );
    addTearDown(client.close);
    final downloader = HttpVideoFileDownloader(
      client,
      resolver,
      timeouts: const VideoDownloadTimeouts(
        headers: Duration(seconds: 2),
        idle: Duration(seconds: 2),
        total: Duration(seconds: 3),
      ),
    );

    await expectLater(
      downloader
          .download(
            Uri.parse('http://media.example/video.mp4'),
            'unused.mp4',
            maxBytes: 1024,
          )
          .timeout(const Duration(seconds: 1)),
      throwsA(isA<AppFailure>()),
    );

    expect(lookups, 2);
    expect(connectionStarted, isFalse);
  });
}
