import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

import '../support/fake_raw_secure_socket.dart';

void main() {
  test('closes the input stream when the secure transport closes reads',
      () async {
    final raw = FakeRawSecureSocket();
    final socket = RawSecureSocketAdapter(raw);
    addTearDown(socket.destroy);
    final completed = Completer<void>();
    socket.listen((_) {}, onDone: completed.complete);

    raw.events.add(RawSocketEvent.readClosed);

    await completed.future;
  });
}
