import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

import '../support/fake_raw_secure_socket.dart';

void main() {
  test('stops draining raw bytes when the consumer pauses synchronously',
      () async {
    final raw = FakeRawSecureSocket()
      ..reads.add(Uint8List.fromList([1]))
      ..reads.add(Uint8List.fromList([2]));
    final socket = RawSecureSocketAdapter(raw);
    addTearDown(socket.destroy);
    late StreamSubscription<Uint8List> subscription;
    final received = <int>[];
    subscription = socket.listen((bytes) {
      received.addAll(bytes);
      if (received.length == 1) subscription.pause();
    });

    raw.events.add(RawSocketEvent.read);

    expect(received, [1]);
    expect(raw.readCalls, 1);
    subscription.resume();
    raw.events.add(RawSocketEvent.read);

    expect(received, [1, 2]);
    await subscription.cancel();
  });
}
