import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('parallel byte evidence requires both transfers to complete', () async {
    final origin = await ProgressiveDeviceOrigin.start(
      responseChunkBytes: 1024,
      pacing: const ProgressiveOriginPacing.perResponseDelay(
        Duration(milliseconds: 5),
      ),
    );
    final client = HttpClient();
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });

    final transfers = [
      _range(client, origin, 'next'),
      _range(client, origin, 'third'),
    ];
    await _waitForConcurrentBytes(origin);

    expect(origin.rangedByteOverlap({'/next.mp4', '/third.mp4'}), isNull);
    await Future.wait(transfers);
    expect(origin.rangedByteOverlap({'/next.mp4', '/third.mp4'}), isNotNull);
  });
}

Future<void> _waitForConcurrentBytes(ProgressiveDeviceOrigin origin) async {
  final watch = Stopwatch()..start();
  while (watch.elapsed.inSeconds < 1) {
    if (origin.bytesServed('next') > 2048 &&
        origin.bytesServed('third') > 2048) {
      return;
    }
    await Future<void>.delayed(const Duration(milliseconds: 5));
  }
  fail('Both ranged transfers did not begin.');
}

Future<void> _range(
  HttpClient client,
  ProgressiveDeviceOrigin origin,
  String id,
) async {
  final request = await client.getUrl(origin.urlFor(id));
  request.headers.set(HttpHeaders.rangeHeader, 'bytes=0-65535');
  await (await request.close()).drain<void>();
}
