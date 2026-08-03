import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

void main() {
  test('downloads plain HTTP media through the pinned connection', () async {
    final server = await ServerSocket.bind(InternetAddress.loopbackIPv4, 0);
    addTearDown(server.close);
    server.listen((socket) {
      socket.listen((_) {
        socket.add(utf8.encode(
          'HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc',
        ));
        socket.close();
      });
    });
    final directory = await Directory.systemTemp.createTemp('ghostr-http-');
    addTearDown(() => directory.delete(recursive: true));
    final destination = File('${directory.path}/video.mp4');
    final resolver = PublicMediaAddressResolver(
      lookup: (_) async => [InternetAddress('8.8.8.8')],
    );
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startConnect: (_, __) => Socket.startConnect(
          InternetAddress.loopbackIPv4,
          server.port,
        ),
      ),
    );
    addTearDown(client.close);

    await HttpVideoFileDownloader(client, resolver).download(
      Uri.parse('http://media.example/video.mp4'),
      destination.path,
      maxBytes: 3,
    );

    expect(await destination.readAsString(), 'abc');
  });
}
