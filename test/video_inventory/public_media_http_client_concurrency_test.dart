import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';
import 'package:http/http.dart' as http;

void main() {
  test('limits concurrent connection attempts and cancels the scheduler',
      () async {
    final resolver = PublicMediaAddressResolver(
      lookup: (_) async => [
        InternetAddress('8.8.8.8'),
        InternetAddress('1.1.1.1'),
        InternetAddress('9.9.9.9'),
        InternetAddress('208.67.222.222'),
      ],
    );
    var started = 0;
    var active = 0;
    var mostActive = 0;
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startConnect: (address, port) async {
          started += 1;
          active += 1;
          if (active > mostActive) mostActive = active;
          final pending = Completer<Socket>();
          return ConnectionTask.fromSocket(pending.future, () {
            active -= 1;
            pending.completeError(const SocketException('cancelled'));
          });
        },
        connectionTimeout: const Duration(milliseconds: 600),
      ),
    );
    addTearDown(client.close);

    await expectLater(
      client.get(Uri.parse('http://media.example/video.mp4')),
      throwsA(isA<http.ClientException>()),
    );
    await Future<void>.delayed(const Duration(milliseconds: 300));

    expect(started, 2);
    expect(mostActive, 2);
    expect(active, 0);
  });
}
