import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('a duplicate path is not held by the first-chunk rendezvous', () async {
    final origin = await ProgressiveDeviceOrigin.start();
    final client = HttpClient();
    final rendezvous = origin.rendezvousFirstChunks({
      '/next.mp4',
      '/third.mp4',
    });
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });

    final first = _range(client, origin, 'next');
    await _waitForBytes(origin, 'next');
    final duplicate = _range(client, origin, 'next');
    try {
      await duplicate.timeout(const Duration(seconds: 1));
    } finally {
      rendezvous.release();
      await Future.wait([first, duplicate]);
    }

    expect(rendezvous.arrivedPaths, {'/next.mp4'});
  });
}

Future<void> _waitForBytes(ProgressiveDeviceOrigin origin, String id) async {
  final watch = Stopwatch()..start();
  while (origin.bytesServed(id) == 0 && watch.elapsed.inSeconds < 1) {
    await Future<void>.delayed(const Duration(milliseconds: 10));
  }
  expect(origin.bytesServed(id), greaterThan(0));
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
