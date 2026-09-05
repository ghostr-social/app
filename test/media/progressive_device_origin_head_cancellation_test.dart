import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('blocked HEAD evidence clears only when its peer closes', () async {
    final origin = await ProgressiveDeviceOrigin.start();
    final client = HttpClient();
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });
    final request = await client.headUrl(origin.urlFor('next'));
    final response = request.close().then<void>((_) {}, onError: (_) {});
    final wait = Stopwatch()..start();
    while (origin.requests.isEmpty && wait.elapsedMilliseconds < 1000) {
      await Future<void>.delayed(const Duration(milliseconds: 5));
    }
    expect(origin.headsRemainBlocked, isTrue);
    request.abort();
    await response;
    await Future<void>.delayed(const Duration(milliseconds: 100));

    expect(origin.headsRemainBlocked, isFalse);
    expect(origin.requests.single.isPeerClosed, isTrue);
    expect(origin.requests.single.servedBytes, 0);
  });
}
