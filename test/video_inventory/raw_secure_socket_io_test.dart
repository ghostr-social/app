import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

import '../support/fake_raw_secure_socket.dart';

void main() {
  test('forwards every Socket sink operation to the secure transport',
      () async {
    final raw = FakeRawSecureSocket();
    final socket = RawSecureSocketAdapter(raw);
    addTearDown(socket.destroy);

    expect(socket.encoding, utf8);
    socket.encoding = latin1;
    socket.add([65]);
    socket.write('é');
    socket.writeAll([1, 2], ',');
    socket.writeCharCode(33);
    socket.writeln('x');
    await socket.addStream(Stream.value([90]));
    await socket.flush();

    expect(raw.written, [65, 233, 49, 44, 50, 33, 120, 10, 90]);
    expect(() => socket.addError(StateError('nope')), throwsUnsupportedError);
    final done = socket.done;
    await socket.close();
    await done;
    expect(raw.shutdownDirection, SocketDirection.send);
  });
}
