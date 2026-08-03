import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

import '../support/fake_raw_secure_socket.dart';

void main() {
  test('resumes a blocked write only after the transport is writable',
      () async {
    final raw = FakeRawSecureSocket()..maximumWriteBytes = 0;
    final socket = RawSecureSocketAdapter(raw);
    addTearDown(socket.destroy);
    socket.add([1, 2, 3]);

    final flush = socket.flush();
    await Future<void>.delayed(Duration.zero);
    expect(raw.writeCalls, 1);
    expect(raw.writeEventsEnabled, isTrue);

    raw.maximumWriteBytes = 3;
    raw.events.add(RawSocketEvent.write);
    await flush;
    expect(raw.written, [1, 2, 3]);
    expect(raw.writeEventsEnabled, isFalse);
  });
}
