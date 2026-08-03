import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

import '../support/fake_raw_secure_socket.dart';

void main() {
  test('forwards Socket metadata and options to the secure transport',
      () async {
    final raw = FakeRawSecureSocket();
    final socket = RawSecureSocketAdapter(raw);
    final rawOption = RawSocketOption(1, 2, Uint8List(4));

    expect(socket.address, InternetAddress.loopbackIPv4);
    expect(socket.port, 1234);
    expect(socket.remoteAddress, InternetAddress('8.8.8.8'));
    expect(socket.remotePort, 443);
    expect(socket.setOption(SocketOption.tcpNoDelay, true), isTrue);
    expect(raw.configuredOption, SocketOption.tcpNoDelay);
    expect(socket.getRawOption(rawOption), Uint8List.fromList([7]));
    socket.setRawOption(rawOption);
    expect(raw.configuredRawOption, same(rawOption));

    socket.destroy();
    socket.destroy();
    await socket.close();
    expect(raw.closeCalls, 1);
  });
}
