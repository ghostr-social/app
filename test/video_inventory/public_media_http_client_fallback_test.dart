import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/http_video_file_downloader.dart';
import 'package:ghostr/platform/media/video_download_timeouts.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

void main() {
  test('uses a later validated address while the first remains pending',
      () async {
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
    final firstSocket = Completer<Socket>();
    var firstCancelled = false;
    final resolver = PublicMediaAddressResolver(
      lookup: (_) async => [
        InternetAddress('8.8.8.8'),
        InternetAddress('8.8.8.8'),
        InternetAddress('1.1.1.1'),
      ],
    );
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startConnect: (address, _) async {
          if (address.address == '1.1.1.1') {
            return Socket.startConnect(
              InternetAddress.loopbackIPv4,
              server.port,
            );
          }
          return ConnectionTask.fromSocket(firstSocket.future, () {
            firstCancelled = true;
            firstSocket.completeError(const SocketException('cancelled'));
          });
        },
      ),
    );
    addTearDown(client.close);
    final directory = await Directory.systemTemp.createTemp('ghostr-fallback-');
    addTearDown(() => directory.delete(recursive: true));
    final destination = File('${directory.path}/video.mp4');

    await HttpVideoFileDownloader(
      client,
      resolver,
      timeouts: const VideoDownloadTimeouts(
        headers: Duration(seconds: 1),
        idle: Duration(seconds: 1),
        total: Duration(seconds: 2),
      ),
    ).download(
      Uri.parse('http://media.example/video.mp4'),
      destination.path,
      maxBytes: 3,
    );

    expect(await destination.readAsString(), 'abc');
    expect(firstCancelled, isTrue);
  });
}
