import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_address_resolver.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

void main() {
  test('tries the next validated address after an immediate failure', () async {
    final server = await ServerSocket.bind(InternetAddress.loopbackIPv4, 0);
    addTearDown(server.close);
    server.listen((socket) {
      socket.listen((_) {
        socket.add(utf8.encode(
          'HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok',
        ));
        socket.close();
      });
    });
    final resolver = PublicMediaAddressResolver(
      lookup: (_) async => [
        InternetAddress('8.8.8.8'),
        InternetAddress('1.1.1.1'),
      ],
    );
    final attempted = <String>[];
    final client = createPublicMediaHttpClient(
      resolver,
      config: PublicMediaHttpClientConfig(
        startConnect: (address, _) async {
          attempted.add(address.address);
          if (address.address == '8.8.8.8') {
            throw const SocketException('unreachable');
          }
          return Socket.startConnect(InternetAddress.loopbackIPv4, server.port);
        },
      ),
    );
    addTearDown(client.close);

    final response = await client.get(
      Uri.parse('http://media.example/video.mp4'),
    );

    expect(response.body, 'ok');
    expect(attempted, ['8.8.8.8', '1.1.1.1']);
  });
}
