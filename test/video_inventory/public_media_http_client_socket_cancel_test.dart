import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';
import 'package:http/http.dart' as http;

void main() {
  test('closes a socket that connects after the request was cancelled',
      () async {
    final server = await ServerSocket.bind(InternetAddress.loopbackIPv4, 0);
    addTearDown(server.close);
    final peerClosed = Completer<void>();
    server.listen((socket) {
      socket.listen((_) {}, onDone: peerClosed.complete);
    });
    final lateSocket = await Socket.connect(
      InternetAddress.loopbackIPv4,
      server.port,
    );
    final pendingSocket = Completer<Socket>();
    var taskCancelled = false;
    final resolver = PublicMediaAddressResolver(
      lookup: (_) async => [InternetAddress('8.8.8.8')],
    );
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startConnect: (_, __) async => ConnectionTask.fromSocket(
          pendingSocket.future,
          () => taskCancelled = true,
        ),
        connectionTimeout: const Duration(milliseconds: 100),
      ),
    );
    addTearDown(client.close);

    await expectLater(
      client.get(Uri.parse('http://media.example/video.mp4')),
      throwsA(isA<http.ClientException>()),
    );
    pendingSocket.complete(lateSocket);

    expect(taskCancelled, isTrue);
    await peerClosed.future.timeout(const Duration(seconds: 1));
  });
}
