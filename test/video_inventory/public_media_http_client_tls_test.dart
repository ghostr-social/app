import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

void main() {
  test('keeps TLS enabled when connecting to a pinned address', () async {
    final server = await ServerSocket.bind(InternetAddress.loopbackIPv4, 0);
    addTearDown(server.close);
    final firstBytes = Completer<List<int>>();
    server.listen((socket) {
      socket.listen((bytes) {
        if (!firstBytes.isCompleted) firstBytes.complete(bytes);
        socket.destroy();
      });
    });
    final resolver = PublicMediaAddressResolver(
      lookup: (_) async => [InternetAddress('8.8.8.8')],
    );
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startRawConnect: (_, __) => MediaRawSocketTask.startConnect(
          InternetAddress.loopbackIPv4,
          server.port,
        ),
      ),
    );
    addTearDown(client.close);

    await expectLater(
      HttpVideoFileDownloader(client, resolver).download(
        Uri.parse('https://media.example/video.mp4'),
        'unused.mp4',
        maxBytes: 1024,
      ),
      throwsA(isA<AppFailure>()),
    );

    expect((await firstBytes.future).first, 0x16);
  });
}
