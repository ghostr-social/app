import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';
import 'package:http/http.dart' as http;

import '../support/fake_media_raw_socket_task.dart';

void main() {
  test('cancels a raw connection task returned after HTTPS times out',
      () async {
    final resolver = PublicMediaAddressResolver(
      lookup: (_) async => [InternetAddress('8.8.8.8')],
    );
    final start = Completer<MediaRawSocketTask>();
    final pendingRaw = Completer<RawSocket>();
    var taskCancelled = false;
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startRawConnect: (_, __) => start.future,
        connectionTimeout: const Duration(milliseconds: 100),
      ),
    );
    addTearDown(client.close);

    await expectLater(
      client.get(Uri.parse('https://media.example/video.mp4')),
      throwsA(isA<http.ClientException>()),
    );
    start.complete(FakeMediaRawSocketTask(pendingRaw.future, () {
      taskCancelled = true;
      pendingRaw.completeError(const SocketException('cancelled'));
    }));
    await Future<void>.delayed(Duration.zero);

    expect(taskCancelled, isTrue);
  });
}
