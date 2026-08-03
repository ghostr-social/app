import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';
import 'package:http/http.dart' as http;

void main() {
  test('cancels a connection task returned after the request timeout',
      () async {
    final resolver = PublicMediaAddressResolver(
      lookup: (_) async => [InternetAddress('8.8.8.8')],
    );
    final start = Completer<ConnectionTask<Socket>>();
    final pendingSocket = Completer<Socket>();
    var taskCancelled = false;
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startConnect: (_, __) => start.future,
        connectionTimeout: const Duration(milliseconds: 100),
      ),
    );
    addTearDown(client.close);

    await expectLater(
      client.get(Uri.parse('http://media.example/video.mp4')),
      throwsA(isA<http.ClientException>()),
    );
    start.complete(ConnectionTask.fromSocket(pendingSocket.future, () {
      taskCancelled = true;
      pendingSocket.completeError(const SocketException('cancelled'));
    }));
    await Future<void>.delayed(Duration.zero);

    expect(taskCancelled, isTrue);
  });
}
