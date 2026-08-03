import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/network/public_media_http_client.dart';

import '../support/fake_raw_secure_socket.dart';

void main() {
  test('fails pending reads and writes when the secure transport fails',
      () async {
    final raw = FakeRawSecureSocket()..maximumWriteBytes = 0;
    final socket = RawSecureSocketAdapter(raw);
    addTearDown(socket.destroy);
    final errors = <Object>[];
    final subscription = socket.listen(
      (_) {},
      onError: errors.add,
    );
    addTearDown(subscription.cancel);
    socket.add([1]);
    final flush = socket.flush();
    await Future<void>.delayed(Duration.zero);

    raw.events.addError(const SocketException('broken'));

    await expectLater(flush, throwsA(isA<SocketException>()));
    expect(errors, [isA<SocketException>()]);
  });
}
