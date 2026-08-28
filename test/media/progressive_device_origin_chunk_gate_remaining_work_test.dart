import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/progressive_device_origin.dart';

void main() {
  test('chunk gate only holds a request with bytes remaining', () async {
    final origin = await ProgressiveDeviceOrigin.start();
    final client = HttpClient();
    final gate = origin.holdAfterChunks({
      '/short.mp4',
      '/long.mp4',
    }, afterChunks: 2);
    addTearDown(() async {
      client.close(force: true);
      await origin.close();
    });

    await _range(
      client,
      origin,
      'short',
      32768,
    ).timeout(const Duration(seconds: 1));
    expect(gate.isReached, isFalse);
    final longer = _range(client, origin, 'long', 49152);
    await gate.reached.timeout(const Duration(seconds: 1));

    expect(gate.path, '/long.mp4');
    expect(gate.requestRange, (start: 0, end: 49152));
    gate.release();
    await longer;
  });
}

Future<void> _range(
  HttpClient client,
  ProgressiveDeviceOrigin origin,
  String id,
  int bytes,
) async {
  final request = await client.getUrl(origin.urlFor(id));
  request.headers.set(HttpHeaders.rangeHeader, 'bytes=0-${bytes - 1}');
  await (await request.close()).drain<void>();
}
