import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('pre-body gate observes an exact client close before release', () async {
    final origin = await ProgressiveDeviceOrigin.start(
      responseChunkBytes: 1024,
    );
    final client = HttpClient();
    final gate = origin.holdBeforeFirstBody({
      '/next.mp4',
    }, timeout: const Duration(seconds: 5));
    addTearDown(() async {
      gate.release();
      client.close(force: true);
      await origin.close();
    });

    final request = await client.getUrl(origin.urlFor('next'));
    request.headers.set(HttpHeaders.rangeHeader, 'bytes=0-65535');
    final response = await request.close();
    var receivedBytes = 0;
    final body = response.listen((bytes) => receivedBytes += bytes.length);
    await gate.reached.timeout(const Duration(seconds: 1));

    await body.cancel();
    client.close(force: true);
    await gate.peerClosed.timeout(const Duration(seconds: 1));
    expect(receivedBytes, 0);
    expect(origin.requestsFor('next').single.servedBytes, 0);

    gate.release();
    await _waitForTerminal(origin.requestsFor('next').single);
    expect(
      origin.requestsFor('next').single.outcome,
      ProgressiveOriginRequestOutcome.clientCanceled,
    );
    expect(
      origin.requestsFor('next').single.servedBytes,
      lessThanOrEqualTo(1024),
    );
  });
}

Future<void> _waitForTerminal(ProgressiveOriginRequest request) async {
  for (var attempt = 0; attempt < 40; attempt += 1) {
    if (request.outcome != ProgressiveOriginRequestOutcome.serving) return;
    await Future<void>.delayed(const Duration(milliseconds: 25));
  }
  fail('Canceled request never became terminal.');
}
