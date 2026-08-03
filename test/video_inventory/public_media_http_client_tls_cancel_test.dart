import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';
import 'package:http/http.dart' as http;

void main() {
  test('connection timeout closes a socket stalled in the TLS handshake',
      () async {
    final server = await ServerSocket.bind(InternetAddress.loopbackIPv4, 0);
    addTearDown(server.close);
    final accepted = Completer<void>();
    final peerClosed = Completer<void>();
    server.listen((socket) {
      accepted.complete();
      socket.listen(
        (_) {},
        onDone: peerClosed.complete,
      );
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
        connectionTimeout: const Duration(milliseconds: 100),
      ),
    );
    addTearDown(client.close);

    await expectLater(
      client.get(Uri.parse('https://media.example/video.mp4')),
      throwsA(isA<http.ClientException>()),
    );
    await accepted.future.timeout(const Duration(seconds: 1));

    await peerClosed.future.timeout(const Duration(seconds: 1));
  });
}
