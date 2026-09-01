import 'dart:async';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('pre-body gate exposes headers while holding every body byte', () async {
    final origin = await ProgressiveDeviceOrigin.start();
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
    final responseFuture = request.close();
    await gate.reached.timeout(const Duration(seconds: 1));

    final response = await responseFuture.timeout(const Duration(seconds: 1));
    var receivedBytes = 0;
    final bodyDone = Completer<void>();
    response.listen(
      (bytes) => receivedBytes += bytes.length,
      onDone: bodyDone.complete,
    );
    await Future<void>.delayed(const Duration(milliseconds: 50));
    expect(response.statusCode, HttpStatus.partialContent);
    expect(receivedBytes, 0);

    gate.release();
    await bodyDone.future.timeout(const Duration(seconds: 2));
    expect(receivedBytes, 64 * 1024);
  });
}
